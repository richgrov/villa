#include <chrono>
#include <thread>

#include "net/networking.h"

using namespace simulo;

int main() {
   Networking net(25565);
   net.listen();

   while (true) {
      net.poll();
      std::this_thread::sleep_for(std::chrono::milliseconds(20));
   }

   return 0;
}
