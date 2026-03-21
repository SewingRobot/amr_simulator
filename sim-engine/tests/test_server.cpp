#include "grpc/telemetry_server.h"
#include "grpc/command_server.h"

#include <gtest/gtest.h>
#include <nlohmann/json.hpp>

#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <cstring>
#include <string>
#include <thread>
#include <chrono>

using namespace amr::sim;

// Helper: connect to localhost on the given port.  Retries briefly to allow
// the server to start accepting.
static int connectToLocalhost(int port, int max_retries = 10) {
    for (int attempt = 0; attempt < max_retries; ++attempt) {
        int fd = ::socket(AF_INET, SOCK_STREAM, 0);
        if (fd < 0) return -1;

        sockaddr_in addr{};
        addr.sin_family = AF_INET;
        addr.sin_port = htons(static_cast<uint16_t>(port));
        ::inet_pton(AF_INET, "127.0.0.1", &addr.sin_addr);

        if (::connect(fd, reinterpret_cast<sockaddr*>(&addr), sizeof(addr)) == 0) {
            return fd;
        }
        ::close(fd);
        std::this_thread::sleep_for(std::chrono::milliseconds(50));
    }
    return -1;
}

// Helper: read a full newline-delimited line from a socket.
static std::string recvLine(int fd, int timeout_ms = 2000) {
    struct timeval tv;
    tv.tv_sec = timeout_ms / 1000;
    tv.tv_usec = (timeout_ms % 1000) * 1000;
    ::setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));

    std::string buf;
    char c;
    while (true) {
        ssize_t n = ::recv(fd, &c, 1, 0);
        if (n <= 0) break;
        if (c == '\n') break;
        buf += c;
    }
    return buf;
}

// ─── TelemetryServer tests ──────────────────────────────────────────

TEST(TelemetryServer, AcceptsConnectionAndBroadcasts) {
    const int port = 18051;
    TelemetryServer server(port);
    server.start();

    int client = connectToLocalhost(port);
    ASSERT_GE(client, 0) << "Failed to connect to TelemetryServer";

    // Give the server a moment to register the client
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    EXPECT_EQ(server.clientCount(), 1u);

    // Broadcast some telemetry
    std::vector<RobotTelemetry> telemetry;
    RobotTelemetry t;
    t.robot_id = "r1";
    t.timestamp_ms = 1000.0;
    t.sequence_number = 42;
    t.pose = {1.0, 2.0, 0.5};
    t.velocity = {0.3, 0.1};
    t.battery_percent = 87.5;
    t.status = "busy";
    telemetry.push_back(t);

    server.broadcastTelemetry(telemetry);

    // Read the line from the client
    std::string line = recvLine(client);
    ASSERT_FALSE(line.empty()) << "Received empty telemetry message";

    auto j = nlohmann::json::parse(line);
    ASSERT_TRUE(j.is_array());
    ASSERT_EQ(j.size(), 1u);

    auto& r = j[0];
    EXPECT_EQ(r["robot_id"], "r1");
    EXPECT_DOUBLE_EQ(r["pose"]["x"].get<double>(), 1.0);
    EXPECT_DOUBLE_EQ(r["pose"]["y"].get<double>(), 2.0);
    EXPECT_DOUBLE_EQ(r["pose"]["theta"].get<double>(), 0.5);
    EXPECT_DOUBLE_EQ(r["velocity"]["linear"].get<double>(), 0.3);
    EXPECT_DOUBLE_EQ(r["velocity"]["angular"].get<double>(), 0.1);
    EXPECT_DOUBLE_EQ(r["battery_percent"].get<double>(), 87.5);
    EXPECT_EQ(r["status"], "busy");

    ::close(client);
    server.stop();
}

TEST(TelemetryServer, MultipleClients) {
    const int port = 18052;
    TelemetryServer server(port);
    server.start();

    int c1 = connectToLocalhost(port);
    int c2 = connectToLocalhost(port);
    ASSERT_GE(c1, 0);
    ASSERT_GE(c2, 0);

    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    EXPECT_EQ(server.clientCount(), 2u);

    std::vector<RobotTelemetry> telemetry;
    RobotTelemetry t;
    t.robot_id = "r2";
    t.pose = {0.0, 0.0, 0.0};
    t.velocity = {0.0, 0.0};
    telemetry.push_back(t);

    server.broadcastTelemetry(telemetry);

    std::string line1 = recvLine(c1);
    std::string line2 = recvLine(c2);
    EXPECT_FALSE(line1.empty());
    EXPECT_FALSE(line2.empty());

    // Both clients should receive identical data
    EXPECT_EQ(line1, line2);

    ::close(c1);
    ::close(c2);
    server.stop();
}

// ─── CommandServer tests ────────────────────────────────────────────

TEST(CommandServer, ReceivesCommandAndResponds) {
    const int port = 18053;
    CommandServer server(port);

    server.setHandler([](const SimCommand& cmd) -> std::string {
        nlohmann::json resp;
        resp["received_type"] = cmd.type;
        resp["received_robot_id"] = cmd.robot_id;
        return resp.dump();
    });

    server.start();

    int client = connectToLocalhost(port);
    ASSERT_GE(client, 0) << "Failed to connect to CommandServer";

    // Send a command
    std::string cmd = R"({"type":"spawn_robot","robot_id":"r1","x":1.0,"y":2.0})" "\n";
    ::send(client, cmd.data(), cmd.size(), 0);

    // Read response
    std::string resp = recvLine(client);
    ASSERT_FALSE(resp.empty());

    auto j = nlohmann::json::parse(resp);
    EXPECT_EQ(j["received_type"], "spawn_robot");
    EXPECT_EQ(j["received_robot_id"], "r1");

    ::close(client);
    server.stop();
}

TEST(CommandServer, HandlesMultipleCommands) {
    const int port = 18054;
    int count = 0;
    CommandServer server(port);

    server.setHandler([&count](const SimCommand& cmd) -> std::string {
        count++;
        return R"({"ok":true})";
    });

    server.start();

    int client = connectToLocalhost(port);
    ASSERT_GE(client, 0);

    // Send three commands in rapid succession
    for (int i = 0; i < 3; ++i) {
        std::string cmd = R"({"type":"send_command","robot_id":"r1","linear":0.5,"angular":0.0})" "\n";
        ::send(client, cmd.data(), cmd.size(), 0);
        // Read response to avoid buffering issues
        recvLine(client);
    }

    EXPECT_EQ(count, 3);

    ::close(client);
    server.stop();
}

// ─── RobotTelemetry JSON serialization ──────────────────────────────

TEST(RobotTelemetry, JsonRoundtrip) {
    RobotTelemetry t;
    t.robot_id = "test_bot";
    t.timestamp_ms = 123456.0;
    t.sequence_number = 99;
    t.pose = {3.14, 2.71, 1.41};
    t.velocity = {0.5, -0.2};
    t.battery_percent = 42.0;
    t.status = "idle";

    auto j = t.toJson();

    EXPECT_EQ(j["robot_id"], "test_bot");
    EXPECT_DOUBLE_EQ(j["timestamp_ms"].get<double>(), 123456.0);
    EXPECT_EQ(j["sequence_number"].get<uint64_t>(), 99u);
    EXPECT_DOUBLE_EQ(j["pose"]["x"].get<double>(), 3.14);
    EXPECT_DOUBLE_EQ(j["pose"]["y"].get<double>(), 2.71);
    EXPECT_DOUBLE_EQ(j["pose"]["theta"].get<double>(), 1.41);
    EXPECT_DOUBLE_EQ(j["velocity"]["linear"].get<double>(), 0.5);
    EXPECT_DOUBLE_EQ(j["velocity"]["angular"].get<double>(), -0.2);
    EXPECT_DOUBLE_EQ(j["battery_percent"].get<double>(), 42.0);
    EXPECT_EQ(j["status"], "idle");
}
