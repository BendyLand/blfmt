#include <stdbool.h>
#include <string.h>
#include <stdio.h>
#include "todo.h"

TodoList* todo_new(size_t capacity)
{
	TodoList* result = (TodoList*)malloc(sizeof(TodoList));
	result->size = 0;
	result->capacity = capacity > 0 ? capacity : 1;
	result->tasks = (Task**)malloc(capacity * sizeof(Task*));
	return result;
}

size_t todo_free(TodoList** todo)
{
	if (todo && (*todo)) {
		if ((*todo)->tasks) {
			for (size_t i = 0; i < (*todo)->size; i++) {
				if ((*todo)->tasks[i]) {
					task_free(&(*todo)->tasks[i]);
				}
			}
			free((*todo)->tasks);
			(*todo)->tasks = NULL;
		}
		free(*todo);
		(*todo) = NULL;
		return 0;
	}
	return 1;
}

void todo_append(TodoList* todo, Task* task)
{
	if ((todo->size+1) >= todo->capacity) {
		todo->capacity = todo->capacity > 0 ? todo->capacity * 2 : 1;
		Task** temp = realloc(todo->tasks, (todo->capacity * sizeof(Task*)));
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

void todo_remove(TodoList* todo, Flag f, size_t id)
{
	bool shift = false;
	size_t largest = get_last_task_id(todo);
	if (id > largest || id < 1) {
		fprintf(stderr, "Invalid task id.\n");
		return;
	}
	for (size_t i = 0; i < todo->size-1; i++) {
		size_t current_id = todo->tasks[i]->id;
		if (current_id == id) shift = true;
		else if (current_id > id && shift == false) {
			fprintf(stderr, "Invalid task id.\n");
			return;
		}
		if (shift) {
			if (current_id == id) task_free(&todo->tasks[i]);
			todo->tasks[i] = todo->tasks[i+1];
		}
	}
    if (id == largest) task_free(&todo->tasks[todo->size - 1]); 
	todo->size--;
	todo_fix_ids(todo);
}

char* todo_to_csv(const TodoList* todo)
{
	if (!todo) {
		fprintf(stderr, "Don't call this on NULL.\n");
		exit(1);
	}
    if (todo->size == 0) {
    	char* result = (char*)malloc(1);
    	result[0] = '\0';
    	return result;
    }
    // Calculate total buffer size
    size_t buffer_size = 0;
    for (size_t i = 0; i < todo->size; i++) {
        Task* task = todo->tasks[i];
        if (task) {
            // id: up to 20 chars (for size_t), data: strlen chars, newline: 1 char
            buffer_size += 20 + 1 + strlen(task->data) + 1;
        }
    }
    // Add space for the final null terminator
    buffer_size += 1;
    char* result = (char*)malloc(buffer_size);
    if (!result) {
    	fprintf(stderr, "Unable to allocate memory for csv output.\n");
    	exit(1);
    }
    result[0] = '\0';  // Start with empty string
    for (size_t i = 0; i < todo->size; i++) {
        Task* task = todo->tasks[i];
        if (task) {
            char line[1024];  // Temporary line buffer
            snprintf(line, sizeof(line), "%zu,%s\n", task->id, task->data);
            strncat(result, line, buffer_size - strlen(result) - 1);
        }
    }
    return result;
}

TodoList* csv_to_todo(const char* csv)
{
    if (!csv) {
    	fprintf(stderr, "Don't call this on NULL.\n");
    	exit(1);
	}
    TodoList* todo = todo_new(0);
    const char* line_start = csv;
    while (*line_start) {
        const char* line_end = strchr(line_start, '\n');
        size_t line_len = line_end ? (size_t)(line_end - line_start) : strlen(line_start);
        char* line = (char*)malloc(line_len + 1);
        if (!line) {
            fprintf(stderr, "Memory allocation failed.\n");
            exit(1);
        }
        strncpy(line, line_start, line_len);
        line[line_len] = '\0';
        char* comma = strchr(line, ',');
        if (comma) {
            *comma = '\0';
            size_t id = (size_t)strtoull(line, NULL, 10);
            const char* data = comma + 1;
            Task* task = task_new(id, data);
            todo_append(todo, task);
        }
        free(line);
        if (!line_end) break;
        line_start = line_end + 1;
    }
    return todo;
}

void todo_fix_ids(TodoList* todo)
{
	for (size_t i = 0; i < todo->size; i++) {
		todo->tasks[i]->id = i+1;
	}
}

size_t get_last_task_id(TodoList* todo)
{
	if (todo->size == 0) return 1;
	return todo->tasks[todo->size-1]->id;
}

size_t get_next_task_id(TodoList* todo)
{
	if (todo->size == 0) return 1;
	return todo->tasks[todo->size-1]->id+1;
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

