#include <stdio.h>

void example();

int main(void)
{
	example();
	return 0;
}

void example()
{
	int test = 5;
	if (test == 2) {
		if (0) {
			// nested if branch
		}
		else {
			// nested "else" branch
		}
		// if branch
	}
	else if (0) {
		// "else" if branch
	}
	else {
		// "else" branch
	}
}

void example_two()
{
	int a;
	switch (5) {
	case 1: 
		// do something
		break;
	case 2: 
		// do something
		break;
	case 3: 
		// do something
		break;
	case 4: 
		// do something
		break;
	case 5: 
		// do something
		break;
	default: 
		// do something
		break;
	}
	int b;
}

void example_three()
{
	int a = 0;
	while (1) {
		a++;
		if (a > 100) break;
	}
}