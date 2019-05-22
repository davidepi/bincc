#ifndef __SETTINGS_HPP__
#define __SETTINGS_HPP__

class Settings
{
public:
    static Settings& instance()
    {
        static Settings instance;
        return instance;
    }

    Settings(Settings const&) = delete;
    void operator=(Settings const&)  = delete;

    const char* working_folder()const;
    int working_folder_len()const;

private:
    Settings();
    ~Settings();

    char* folder;
    int folder_len;

};

#endif
