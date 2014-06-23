
struct ParseResult {
    int success;
};

void init_parser(void);
void parse_query(char *query, struct ParseResult *result);

