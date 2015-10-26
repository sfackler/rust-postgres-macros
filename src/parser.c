#include "postgres.h"
#include "utils/memutils.h"
#include "nodes/nodeFuncs.h"
#include "parser/parser.h"

#include "parser.h"

#define MAX(a, b) (((a) > (b)) ? (a) : (b))

// Postgres internals require this symbol
const char *progname = "rust-postgres-macros";

static bool count_params(Node *node, struct ParseResult *result);

void init_parser(void) {
    MemoryContextInit();
}

void parse_query(char *query, struct ParseResult *result) {
    MemoryContext ctx = AllocSetContextCreate(TopMemoryContext,
                                              "rust-postgres-macros",
                                              ALLOCSET_DEFAULT_MINSIZE,
                                              ALLOCSET_DEFAULT_INITSIZE,
                                              ALLOCSET_DEFAULT_MAXSIZE);
    MemoryContextSwitchTo(ctx);
    List *parsetree;

    PG_TRY();
    {
        parsetree = raw_parser(query);
        result->num_params = 0;
        result->success = 1;
    }
    PG_CATCH();
    {
        ErrorData *error_data = CopyErrorData();
        FlushErrorState();
        result->error_message = malloc(strlen(error_data->message) + 1);
        strcpy(result->error_message, error_data->message);
        result->index = error_data->cursorpos;
        result->success = 0;
    }
    PG_END_TRY();

    if (result->success != 1) {
        MemoryContextSwitchTo(TopMemoryContext);
        MemoryContextDelete(ctx);
        return;
    }

    // raw_expression_tree_walker doesn't support all query types, so mark
    // that we couldn't get the counts.
    PG_TRY();
    {
        count_params((Node *) parsetree, result);
    }
    PG_CATCH();
    {
        FlushErrorState();
        result->num_params = -1;
    }
    PG_END_TRY();

    MemoryContextSwitchTo(TopMemoryContext);
    MemoryContextDelete(ctx);
}

static bool count_params(Node *node, struct ParseResult *result) {
    if (node == NULL) {
        return false;
    }

    if (IsA(node, ParamRef)) {
        ParamRef *param = (ParamRef *) node;
        result->num_params = MAX(result->num_params, param->number);
    }

    return raw_expression_tree_walker(node, count_params, (void *) result);
}
