add_executable(sim_tests
    test_differential_drive.cpp
    test_lidar_2d.cpp
    test_server.cpp
)
target_link_libraries(sim_tests PRIVATE sim_engine_lib GTest::gtest_main)
include(GoogleTest)
gtest_discover_tests(sim_tests)
