#include <cstdlib>
#include <cstring>
#include <cstdio>
#include <unistd.h>
#include "settings.hpp"

#define FOLDER_TEMPLATE "/tmp/bccXXXXXX"

Settings::Settings()
{
    folder = (char*)malloc(sizeof(FOLDER_TEMPLATE)+1);
    strcpy(folder, FOLDER_TEMPLATE);
    if(mkdtemp(folder) == nullptr)
    {
        perror("Could not create temp directory at `/tmp`: ");
        exit(EXIT_FAILURE);
    }
    strcat(folder, "/");
    folder_len = strlen(folder);
}

const char* Settings::working_folder() const
{
    return folder;
}

int Settings::working_folder_len() const
{
    return folder_len;
}

Settings::~Settings()
{
    rmdir(folder);
    free(folder);
}
