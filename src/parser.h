
struct ParseResult {
    int success;
    char *error_message;
    int index;
};

void init_parser(void);
void parse_query(char *query, struct ParseResult *result);

