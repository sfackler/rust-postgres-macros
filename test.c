#include <stdio.h>
#include "postgres.h"
#include "utils/memutils.h"
#include "parser/parser.h"

const char *progname = "rust-postgres-macros";

int main() {
    MemoryContextInit();

    MemoryContext ctx = AllocSetContextCreate(TopMemoryContext,
                                              "rust-postgres-macros",
                                              ALLOCSET_DEFAULT_MINSIZE,
                                              ALLOCSET_DEFAULT_INITSIZE,
                                              ALLOCSET_DEFAULT_MAXSIZE);
    MemoryContextSwitchTo(ctx);

    PG_TRY();
    {
        List *parsetree = raw_parser("SELECT * FROMasdf foo WHERasdf");
        char *str = nodeToString(parsetree);
        printf("%s\n", str);
    }
    PG_CATCH();
    {
        ErrorData *error = CopyErrorData();
        printf("ERROR: %s\n", error->message);
    }
    PG_END_TRY();

    MemoryContextSwitchTo(TopMemoryContext);
    MemoryContextDelete(ctx);
}
