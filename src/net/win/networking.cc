#include "networking.h"

#include <iostream>
#include <string>

#include <MSWSock.h>

using namespace simulo;

namespace {

template <typename T>
std::runtime_error create_func_error(const std::string &func_name, T err_code) {
   auto err_msg = func_name + " failed: " + std::to_string(err_code);
   return std::runtime_error(err_msg);
}

} // namespace

Networking::Networking(std::uint16_t port) : overlapped_() {
   WSAData wsa_data;
   int startup_res = WSAStartup(MAKEWORD(2, 2), &wsa_data);
   if (startup_res != 0) {
      throw create_func_error("WSAStartup", startup_res);
   }

   completion_port_ = CreateIoCompletionPort(INVALID_HANDLE_VALUE, nullptr, 0, 0);
   if (completion_port_ == nullptr) {
      throw create_func_error("CreateIOCompletionPort", GetLastError());
   }

   listen_socket_ = socket(AF_INET, SOCK_STREAM, 0);
   if (listen_socket_ == INVALID_SOCKET) {
      throw create_func_error("socket", WSAGetLastError());
   }

   SOCKADDR_IN bind_addr;
   bind_addr.sin_family = AF_INET;
   bind_addr.sin_addr.s_addr = INADDR_ANY;
   bind_addr.sin_port = htons(port);
   if (bind(listen_socket_, reinterpret_cast<sockaddr *>(&bind_addr), sizeof(bind_addr)) ==
       SOCKET_ERROR) {
      throw create_func_error("bind", WSAGetLastError());
   }
}

void Networking::listen() {
   if (::listen(listen_socket_, 16) == SOCKET_ERROR) {
      throw create_func_error("listen", WSAGetLastError());
   }

   HANDLE listen_port = CreateIoCompletionPort(reinterpret_cast<HANDLE>(listen_socket_),
                                               completion_port_, 0xCafeBabeULL, 0);
   if (listen_port == nullptr) {
      throw create_func_error("CreateIOCompletionPort", GetLastError());
   }

   SOCKET accepted = socket(AF_INET, SOCK_STREAM, 0);

   bool success = AcceptEx(listen_socket_, accepted, accept_buf_, 0, kAddressLen, kAddressLen,
                           nullptr, &overlapped_);
   if (!success && WSAGetLastError() != ERROR_IO_PENDING) {
      abort();
   }
}

void Networking::poll() const {
   DWORD len;
   unsigned long long key;
   WSAOVERLAPPED *overlapped;

   while (true) {
      bool has_messages =
          GetQueuedCompletionStatus(completion_port_, &len, (PULONG_PTR)&key, &overlapped, 0);
      if (!has_messages) {
         break;
      }

      if (key == 0xCafeBabeULL) {
         std::cout << "It worked!\n";
      } else {
         std::cout << "It failed!\n";
      }
   }
}
