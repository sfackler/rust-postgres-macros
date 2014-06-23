
struct ParseResult {
    int success;
    char *error_message;
};

void init_parser(void);
void parse_query(char *query, struct ParseResult *result);

