#include "grpc/telemetry_server.h"

#include <spdlog/spdlog.h>

#include <sys/socket.h>
#include <netinet/in.h>
#include <unistd.h>
#include <fcntl.h>
#include <cerrno>
#include <cstring>
#include <algorithm>

// MSG_NOSIGNAL is not available on macOS; use SO_NOSIGPIPE at socket level instead.
#ifndef MSG_NOSIGNAL
#define MSG_NOSIGNAL 0
#endif

namespace amr::sim {

// ─── RobotTelemetry ─────────────────────────────────────────────────

nlohmann::json RobotTelemetry::toJson() const {
    return nlohmann::json{
        {"robot_id",        robot_id},
        {"timestamp_ms",    timestamp_ms},
        {"sequence_number", sequence_number},
        {"pose", {
            {"x",     pose.x},
            {"y",     pose.y},
            {"theta", pose.theta}
        }},
        {"velocity", {
            {"linear",  velocity.linear},
            {"angular", velocity.angular}
        }},
        {"battery_percent", battery_percent},
        {"status",          status}
    };
}

// ─── TelemetryServer ────────────────────────────────────────────────

TelemetryServer::TelemetryServer(int port)
    : m_port(port) {}

TelemetryServer::~TelemetryServer() {
    stop();
}

void TelemetryServer::start() {
    if (m_running.load()) return;

    // Create TCP socket
    m_serverFd = ::socket(AF_INET, SOCK_STREAM, 0);
    if (m_serverFd < 0) {
        spdlog::error("TelemetryServer: failed to create socket: {}", std::strerror(errno));
        return;
    }

    // Allow address reuse
    int opt = 1;
    ::setsockopt(m_serverFd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));

    // Bind
    sockaddr_in addr{};
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(static_cast<uint16_t>(m_port));

    if (::bind(m_serverFd, reinterpret_cast<sockaddr*>(&addr), sizeof(addr)) < 0) {
        spdlog::error("TelemetryServer: bind failed on port {}: {}", m_port, std::strerror(errno));
        ::close(m_serverFd);
        m_serverFd = -1;
        return;
    }

    if (::listen(m_serverFd, 8) < 0) {
        spdlog::error("TelemetryServer: listen failed: {}", std::strerror(errno));
        ::close(m_serverFd);
        m_serverFd = -1;
        return;
    }

    // Make the listening socket non-blocking so accept() can be interrupted
    int flags = ::fcntl(m_serverFd, F_GETFL, 0);
    ::fcntl(m_serverFd, F_SETFL, flags | O_NONBLOCK);

    m_running = true;
    m_acceptThread = std::thread(&TelemetryServer::acceptLoop, this);

    spdlog::info("TelemetryServer started on port {}", m_port);
}

void TelemetryServer::stop() {
    if (!m_running.load()) return;

    m_running = false;

    if (m_acceptThread.joinable()) {
        m_acceptThread.join();
    }

    // Close all client sockets
    {
        std::lock_guard<std::mutex> lock(m_clientsMutex);
        for (int fd : m_clients) {
            ::close(fd);
        }
        m_clients.clear();
    }

    if (m_serverFd >= 0) {
        ::close(m_serverFd);
        m_serverFd = -1;
    }

    spdlog::info("TelemetryServer stopped");
}

void TelemetryServer::broadcastTelemetry(const std::vector<RobotTelemetry>& telemetry) {
    if (!m_running.load()) return;

    // Build JSON array, one line per broadcast
    nlohmann::json arr = nlohmann::json::array();
    for (const auto& t : telemetry) {
        arr.push_back(t.toJson());
    }
    std::string msg = arr.dump() + "\n";

    std::lock_guard<std::mutex> lock(m_clientsMutex);
    std::vector<int> dead;

    for (int fd : m_clients) {
        ssize_t sent = ::send(fd, msg.data(), msg.size(), MSG_NOSIGNAL);
        if (sent < 0) {
            dead.push_back(fd);
        }
    }

    // Remove dead clients
    for (int fd : dead) {
        ::close(fd);
        m_clients.erase(std::remove(m_clients.begin(), m_clients.end(), fd), m_clients.end());
        spdlog::info("TelemetryServer: client disconnected (fd={})", fd);
    }
}

size_t TelemetryServer::clientCount() const {
    std::lock_guard<std::mutex> lock(m_clientsMutex);
    return m_clients.size();
}

void TelemetryServer::acceptLoop() {
    while (m_running.load()) {
        sockaddr_in client_addr{};
        socklen_t client_len = sizeof(client_addr);
        int client_fd = ::accept(m_serverFd, reinterpret_cast<sockaddr*>(&client_addr), &client_len);

        if (client_fd >= 0) {
#ifdef SO_NOSIGPIPE
            int val = 1;
            ::setsockopt(client_fd, SOL_SOCKET, SO_NOSIGPIPE, &val, sizeof(val));
#endif
            std::lock_guard<std::mutex> lock(m_clientsMutex);
            m_clients.push_back(client_fd);
            spdlog::info("TelemetryServer: client connected (fd={}), total={}", client_fd, m_clients.size());
        } else {
            // Non-blocking: EAGAIN/EWOULDBLOCK means no pending connections
            if (errno == EAGAIN || errno == EWOULDBLOCK) {
                std::this_thread::sleep_for(std::chrono::milliseconds(50));
            }
        }
    }
}

}  // namespace amr::sim
