#include <stdio.h>
#include <stdlib.h>
#include "commands.h"
#include "flags.h"
#include "todo.h"
#include "utils.h"

int run_main(int argc, char** argv);

int main(int argc, char** argv)
{
	return run_main(argc, argv);
}

int run_main(int argc, char** argv)
{
	int res = 0;
	if (argc > 1) {
		Flag flag = parse_flag(argc, argv);
		Command command = parse_command(argc, argv);
		TodoList* todo;
		switch (command) {
		case ERROR:
			switch (flag) {
			case INTERACTIVE:
			case BOTH:
				goto Default;
				break;
			}
			fprintf(stderr, "Invalid command. Printing help menu...\n");
			print_help();
			return 1;
		default:
		Default:
			switch (flag) {
			case LOCAL:
			case BOTH:
				if (!check_for_local_todos()) {
					create_local_file(); 
					todo = todo_new(0);
				}
				else {
					char* filename = get_local_filename();
					char* csv = read_file(filename);
					free(filename);
					todo = csv_to_todo(csv);
					free(csv);
				}
				switch (flag) case BOTH: goto InteractiveRun;
				goto SingleRun;
				break;
			case NONE:
			case INTERACTIVE:
				if (!check_for_global_todos()) {
					create_global_file();  
					todo = todo_new(0);
				}
				else {
					char* filename = get_global_filename();
					char* csv = read_file(filename);
					free(filename);
					todo = csv_to_todo(csv);
					free(csv);
				}
				switch (flag) case NONE: goto SingleRun;
			InteractiveRun:
				res = start_interactive_session(todo, flag); 
				if (todo) todo_free(&todo);
				return res;
			}
			break;
		}
	SingleRun:
		res = run_command(todo, command, flag);
		todo_free(&todo);
	}
	else {
		print_help();
	}
	return res;
}

