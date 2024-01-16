#include <iostream>
#include <sys/ioctl.h>
#include <stdio.h>
#include <termios.h>
#include <unistd.h>
#include <string.h>
#include <cstdlib>
#include <pwd.h>
#include <filesystem>
#include <security/pam_appl.h>

void write_display(std::string &username, std::string &password, winsize w, int c) {
    std::string promptu = "Username: ";
    std::string promptp = "Password: ";
    for(int i = 0; i < (w.ws_row / 2); i++) std::cout << "\n";
    if (c == 1) {
        for(int i = 0; i < (w.ws_col / 2 - promptu.length()); i++) std::cout << " ";
        std::cout << promptu << username;
        std::cout << "\n";
        for(int i = 0; i < (w.ws_col / 2 - promptp.length()); i++) std::cout << " ";
        std::cout << promptp;
        termios oldt;
        tcgetattr(STDIN_FILENO, &oldt);
        termios newt = oldt;
        newt.c_lflag &= ~ECHO;
        tcsetattr(STDIN_FILENO, TCSANOW, &newt);
        std::cin >> password;
        std::cout << "\n";
        tcsetattr(STDIN_FILENO, TCSANOW, &oldt);
    } else {
        for(int i = 0; i < (w.ws_col / 2 - promptu.length()); i++) std::cout << " ";
        std::cout << promptu << username;
        std::cin >> username;
    }
}

int simple_conv(int num_msg, const struct pam_message** msg, struct pam_response** resp, void* appdata_ptr) {
    if (num_msg <= 0 || num_msg > PAM_MAX_NUM_MSG)
        return PAM_CONV_ERR;

    *resp = (struct pam_response*)calloc(num_msg, sizeof(struct pam_response));
    if (!*resp)
        return PAM_BUF_ERR;

    for (int i = 0; i < num_msg; ++i) {
        if (msg[i]->msg_style == PAM_PROMPT_ECHO_OFF) {
            (*resp)[i].resp = strdup((const char*)appdata_ptr);
        } else {
            free((*resp)[i].resp);
            (*resp)[i].resp = nullptr;
        }
    }

    return PAM_SUCCESS;
}

int pam_authenticate(const std::string& username, const std::string& password, std::string start_cmd) {
    const struct pam_conv conv = {simple_conv, (void*)password.c_str()};

    pam_handle_t* pamh = nullptr;
    int retval;

    retval = pam_start("common-auth", username.c_str(), &conv, &pamh);

    if (retval != PAM_SUCCESS) {
        std::cerr << "pam_start failed: " << pam_strerror(pamh, retval) << std::endl;
        return retval;
    }

    retval = pam_authenticate(pamh, 0);

    if (retval != PAM_SUCCESS) {
        std::cerr << "pam_authenticate failed: " << pam_strerror(pamh, retval) << std::endl;
    } else {
        std::cout << "Authentication successful." << std::endl;
        struct passwd *pw = getpwnam(username.c_str());
        std::filesystem::current_path(pw->pw_dir);
        setgid(pw->pw_gid);
        setuid(pw->pw_uid);
        char *env[16];
        char envc[16][64];
        sprintf(env[0]=envc[0],"TERM=xterm");
        sprintf(env[1]=envc[1],"USER=%s",pw->pw_name);
        sprintf(env[2]=envc[2],"HOME=%s",pw->pw_dir);
        sprintf(env[3]=envc[3],"SHELL=%s",pw->pw_shell);
        sprintf(env[4]=envc[4],"LOGNAME=%s",pw->pw_name);
        std::cout << execve(start_cmd.c_str(), NULL, env);
    }

    // Clean up
    pam_end(pamh, PAM_SUCCESS);

    return retval;
}

int main(int argc, char* argv[]) {
    // w.ws_row && w.ws_col
    struct winsize w;
    ioctl(STDOUT_FILENO, TIOCGWINSZ, &w);
    std::string start_cmd;
    switch (argc) {
        case 1:
            start_cmd = "/bin/bash";
            break;
        case 2:
            start_cmd = argv[2];
            break;
        default:
            start_cmd = argv[2];
            break;
    }

    // fancy terminal stuff
    std::string username = "";
    std::string password = "";
    write_display(username, password, w, 0);
    std::cout << "\033[H\033[2J\033[3J";
    write_display(username, password, w, 1);

    int retval = pam_authenticate(username, password, start_cmd);
    return 0;
}