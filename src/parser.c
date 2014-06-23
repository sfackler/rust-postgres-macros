#include "postgres.h"
#include "utils/memutils.h"
#include "parser/parser.h"

#include "parser.h"

// Postgres internals require this symbol
const char *progname = "rust-postgres-macros";

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

    PG_TRY();
    {
        List *parsetree = raw_parser(query);
        result->success = 1;
    }
    PG_CATCH();
    {
        result->success = 0;
    }
    PG_END_TRY();

    MemoryContextSwitchTo(TopMemoryContext);
    MemoryContextDelete(ctx);
}
