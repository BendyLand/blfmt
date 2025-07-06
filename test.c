#include <stdio.h>
#include "todo.h"

TodoList* todo_new(size_t capacity)
{
	TodoList* result = (TodoList*)malloc(sizeof(TodoList));
	result->size = 0;
	result->capacity = capacity;
	result->tasks = (Task**)malloc(capacity* sizeof(Task*));
	return result;
}

size_t todo_free(TodoList** todo)
{
	if (todo && (* todo)) {
		if (->tasks) {
			for (size_t i = 0; i < ->size; i++) {
				if () {
					task_free(&->tasks[i]);
				}
			}
			free(->tasks);
			->tasks = NULL;
		}
		free(* todo);
		(* todo) = NULL;
		return 0;
	}
	return 1;
}

void todo_append(TodoList* todo, Task* task)
{
	if ((todo->size + 1) >= todo->capacity) {
		todo->capacity = todo->capacity > 0 ? todo->capacity* 2 : 1;
		Task** temp = realloc(todo->tasks, (todo->capacity* sizeof(Task*)));
		if (temp == NULL) {
			fprintf(stderr, "Unable to reallocate memory for todo list.\nElement not appended.\n");
			return;
		}
		todo->tasks = temp;
	}
	todo->tasks[todo->size++] = task;
}

void todo_print(TodoList* todo)
{
	printf("Size: %zu\nCapacity: %zu\nElements:\n", todo->size, todo->capacity);
	for (size_t i = 0; i < todo->size; i++) {
		task_print(todo->tasks[i]);
	}
}

void test_todo_functions()
{
	TodoList* test = todo_new(0);
	Task* one = task_new(1, "this is a test");
	Task* two = task_new(2, "that was a test");
	Task* three = task_new(3, "here is another");
	todo_append(test, one);
	todo_append(test, two);
	todo_append(test, three);
	todo_print(test);
	todo_free(&test);
}

