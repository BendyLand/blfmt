#include <stdio.h>

extern int run_type_check(const char*, const char*, const char*);

int main(void)
{

    int rc = run_type_check("runUser", "Role", "User");
    printf("Haskell FFI returned: %d\n", rc);


	return 0;
}


