from conan import ConanFile
from conan.tools.cmake import cmake_layout


class AmrSimEngine(ConanFile):
    settings = "os", "compiler", "build_type", "arch"
    generators = "CMakeDeps", "CMakeToolchain"

    def requirements(self):
        self.requires("spdlog/1.14.1")
        self.requires("nlohmann_json/3.11.3")
        self.requires("gtest/1.15.0")
        self.requires("grpc/1.67.1")
        self.requires("protobuf/5.27.0")

    def layout(self):
        cmake_layout(self)
