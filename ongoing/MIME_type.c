#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>

/*
**	Codingame Puzzle
*/

int main()
{
    // Number of elements which make up the association table.
    int N;
    scanf("%d", &N);
    // Number Q of file names to be analyzed.
    int Q;
    scanf("%d", &Q);
    for (int i = 0; i < N; i++) {
        // file extension
        char EXT[11];
        // MIME type.
        char MT[51];
        scanf("%s%s", EXT, MT); fgetc(stdin);
    }
    for (int i = 0; i < Q; i++) {
        // One file name per line.
        char FNAME[257];
        scanf("%[^\n]", FNAME); fgetc(stdin);
    }

    // Write an answer using printf(). DON'T FORGET THE TRAILING \n
    // To debug: fprintf(stderr, "Debug messages...\n");


    // For each of the Q filenames, display on a line the corresponding MIME type. If there is no corresponding type, then display UNKNOWN.
    printf("UNKNOWN\n");

    return 0;
}
