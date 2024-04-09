#include <chrono>
#include <thread>

#include "net/networking.h"

using namespace simulo;

int main() {
   Networking net(25565);
   net.Listen();

   while (true) {
      net.Poll();
      std::this_thread::sleep_for(std::chrono::milliseconds(20));
   }

   return 0;
}
