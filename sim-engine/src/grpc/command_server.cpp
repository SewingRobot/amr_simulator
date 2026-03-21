#include "grpc/command_server.h"

#include <nlohmann/json.hpp>
#include <spdlog/spdlog.h>

#include <sys/socket.h>
#include <netinet/in.h>
#include <unistd.h>
#include <fcntl.h>
#include <cerrno>
#include <cstring>
#include <sstream>

// MSG_NOSIGNAL is not available on macOS; use SO_NOSIGPIPE at socket level instead.
#ifndef MSG_NOSIGNAL
#define MSG_NOSIGNAL 0
#endif

namespace amr::sim {

// ─── helpers ────────────────────────────────────────────────────────

static SimCommand parseCommand(const nlohmann::json& j) {
    SimCommand cmd;
    cmd.type = j.value("type", "");
    cmd.robot_id = j.value("robot_id", "");
    cmd.x = j.value("x", 0.0);
    cmd.y = j.value("y", 0.0);
    cmd.theta = j.value("theta", 0.0);
    cmd.linear = j.value("linear", 0.0);
    cmd.angular = j.value("angular", 0.0);
    return cmd;
}

// ─── CommandServer ──────────────────────────────────────────────────

CommandServer::CommandServer(int port)
    : m_port(port) {}

CommandServer::~CommandServer() {
    stop();
}

void CommandServer::setHandler(CommandHandler handler) {
    std::lock_guard<std::mutex> lock(m_handlerMutex);
    m_handler = std::move(handler);
}

void CommandServer::start() {
    if (m_running.load()) return;

    m_serverFd = ::socket(AF_INET, SOCK_STREAM, 0);
    if (m_serverFd < 0) {
        spdlog::error("CommandServer: failed to create socket: {}", std::strerror(errno));
        return;
    }

    int opt = 1;
    ::setsockopt(m_serverFd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    sockaddr_in addr{};
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(static_cast<uint16_t>(m_port));

    if (::bind(m_serverFd, reinterpret_cast<sockaddr*>(&addr), sizeof(addr)) < 0) {
        spdlog::error("CommandServer: bind failed on port {}: {}", m_port, std::strerror(errno));
        ::close(m_serverFd);
        m_serverFd = -1;
        return;
    }

    if (::listen(m_serverFd, 8) < 0) {
        spdlog::error("CommandServer: listen failed: {}", std::strerror(errno));
        ::close(m_serverFd);
        m_serverFd = -1;
        return;
    }

    int flags = ::fcntl(m_serverFd, F_GETFL, 0);
    ::fcntl(m_serverFd, F_SETFL, flags | O_NONBLOCK);

    m_running = true;
    m_acceptThread = std::thread(&CommandServer::acceptLoop, this);

    spdlog::info("CommandServer started on port {}", m_port);
}

void CommandServer::stop() {
    if (!m_running.load()) return;

    m_running = false;

    if (m_acceptThread.joinable()) {
        m_acceptThread.join();
    }

    // Wait for client handler threads
    {
        std::lock_guard<std::mutex> lock(m_clientThreadsMutex);
        for (auto& t : m_clientThreads) {
            if (t.joinable()) t.join();
        }
        m_clientThreads.clear();
    }

    if (m_serverFd >= 0) {
        ::close(m_serverFd);
        m_serverFd = -1;
    }

    spdlog::info("CommandServer stopped");
}

void CommandServer::acceptLoop() {
    while (m_running.load()) {
        sockaddr_in client_addr{};
        socklen_t client_len = sizeof(client_addr);
        int client_fd = ::accept(m_serverFd, reinterpret_cast<sockaddr*>(&client_addr), &client_len);

        if (client_fd >= 0) {
            spdlog::info("CommandServer: client connected (fd={})", client_fd);
            std::lock_guard<std::mutex> lock(m_clientThreadsMutex);
            m_clientThreads.emplace_back(&CommandServer::handleClient, this, client_fd);
        } else {
            if (errno == EAGAIN || errno == EWOULDBLOCK) {
                std::this_thread::sleep_for(std::chrono::milliseconds(50));
            }
        }
    }
}

void CommandServer::handleClient(int client_fd) {
#ifdef SO_NOSIGPIPE
    {
        int val = 1;
        ::setsockopt(client_fd, SOL_SOCKET, SO_NOSIGPIPE, &val, sizeof(val));
    }
#endif

    // Set a short read timeout so we can check m_running periodically
    struct timeval tv;
    tv.tv_sec = 0;
    tv.tv_usec = 200000;  // 200ms
    ::setsockopt(client_fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));

    std::string buffer;
    char chunk[4096];

    while (m_running.load()) {
        ssize_t n = ::recv(client_fd, chunk, sizeof(chunk) - 1, 0);
        if (n > 0) {
            chunk[n] = '\0';
            buffer.append(chunk, static_cast<size_t>(n));

            // Process complete lines (newline-delimited JSON)
            size_t pos;
            while ((pos = buffer.find('\n')) != std::string::npos) {
                std::string line = buffer.substr(0, pos);
                buffer.erase(0, pos + 1);

                if (line.empty()) continue;

                try {
                    auto j = nlohmann::json::parse(line);
                    SimCommand cmd = parseCommand(j);
                    spdlog::info("CommandServer: received command type='{}' robot_id='{}'",
                                 cmd.type, cmd.robot_id);

                    std::string response;
                    {
                        std::lock_guard<std::mutex> lock(m_handlerMutex);
                        if (m_handler) {
                            response = m_handler(cmd);
                        } else {
                            response = R"({"error":"no handler registered"})";
                        }
                    }
                    response += "\n";
                    ::send(client_fd, response.data(), response.size(), MSG_NOSIGNAL);
                } catch (const nlohmann::json::exception& e) {
                    spdlog::warn("CommandServer: invalid JSON: {}", e.what());
                    std::string err = R"({"error":"invalid JSON"})" "\n";
                    ::send(client_fd, err.data(), err.size(), MSG_NOSIGNAL);
                }
            }
        } else if (n == 0) {
            // Client disconnected
            break;
        }
        // n < 0 with EAGAIN/EWOULDBLOCK is the timeout; loop continues
    }

    ::close(client_fd);
    spdlog::info("CommandServer: client disconnected (fd={})", client_fd);
}

}  // namespace amr::sim
