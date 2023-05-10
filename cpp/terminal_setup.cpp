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
    #include <stdlib.h>
    #include <unistd.h>
    #include <fcntl.h>
    #include <sys/time.h>
    #include <sys/types.h>
    #include <sys/termios.h>
    #include <sys/mman.h>

    struct termios original_tio;

    void disable_input_buffering() {
        tcgetattr(STDIN_FILENO, &original_tio);
        struct termios new_tio = original_tio;
        new_tio.c_lflag &= ~ICANON & ~ECHO;
        tcsetattr(STDIN_FILENO, TCSANOW, &new_tio);
    }

    void restore_input_buffering() {
        tcsetattr(STDIN_FILENO, TCSANOW, &original_tio);
    }

    extern "C" {
        uint16_t check_key() {
            fd_set readfds;
            FD_ZERO(&readfds);
            FD_SET(STDIN_FILENO, &readfds);

            struct timeval timeout;
            timeout.tv_sec = 0;
            timeout.tv_usec = 0;
            return select(1, &readfds, NULL, NULL, &timeout) != 0;
        }
    }        
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