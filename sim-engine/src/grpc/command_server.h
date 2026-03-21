#pragma once

#include <atomic>
#include <functional>
#include <mutex>
#include <string>
#include <thread>
#include <vector>

namespace amr::sim {

/// A command received over TCP, parsed from JSON.
struct SimCommand {
    std::string type;          // "spawn_robot", "remove_robot", "send_command",
                               // "start_sim", "stop_sim", "get_state"
    std::string robot_id;
    double x = 0.0;
    double y = 0.0;
    double theta = 0.0;
    double linear = 0.0;
    double angular = 0.0;
};

/// Simple TCP server that accepts JSON commands from external clients.
///
/// Each incoming TCP connection is serviced on its own thread.  Commands are
/// newline-delimited JSON objects.  A user-supplied callback is invoked for
/// every valid command, and the callback's return value (a JSON string) is
/// sent back to the client as a response.
class CommandServer {
public:
    /// Callback type: receives a command and returns a JSON response string.
    using CommandHandler = std::function<std::string(const SimCommand& cmd)>;

    explicit CommandServer(int port);
    ~CommandServer();

    /// Set the handler invoked for each received command.
    void setHandler(CommandHandler handler);

    /// Start listening for connections on a background thread.
    void start();

    /// Stop the server and close all connections.
    void stop();

    /// Return the port the server is configured to listen on.
    int port() const { return m_port; }

private:
    void acceptLoop();
    void handleClient(int client_fd);

    int m_port;
    int m_serverFd = -1;
    std::atomic<bool> m_running{false};
    std::thread m_acceptThread;

    mutable std::mutex m_handlerMutex;
    CommandHandler m_handler;

    std::mutex m_clientThreadsMutex;
    std::vector<std::thread> m_clientThreads;
};

}  // namespace amr::sim
