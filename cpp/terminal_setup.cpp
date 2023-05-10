#include <cstdio>
#include <cstdint>
#include <signal.h>

#ifdef _WIN32
    #include <Windows.h>
    #include <conio.h>

    HANDLE hStdin = INVALID_HANDLE_VALUE;
    DWORD fdwMode, fdwOldMode;

    void disable_input_buffering() {
        hStdin = GetStdHandle(STD_INPUT_HANDLE);
        GetConsoleMode(hStdin, &fdwOldMode); 
        fdwMode = fdwOldMode
                ^ ENABLE_ECHO_INPUT  
                ^ ENABLE_LINE_INPUT; 
                
        SetConsoleMode(hStdin, fdwMode);
        FlushConsoleInputBuffer(hStdin); 
    }

    void restore_input_buffering() {
        SetConsoleMode(hStdin, fdwOldMode);
    }

    extern "C" {
        uint16_t check_key() {
            return WaitForSingleObject(hStdin, 1000) == WAIT_OBJECT_0 && _kbhit();
        }
    }
#elif __linux__
#endif

void handle_interrupt(int signal) {
    restore_input_buffering();
    printf("\n");
    exit(-2);
}

extern "C" {
    void setup_term() {
        printf("Setting up terminal...\n");
        signal(SIGINT, handle_interrupt);
        disable_input_buffering();
    }

    void shutdown_term() {
        restore_input_buffering();
        printf("Shutdowning...\n");
    }
}