#include <doctest/doctest.h>

#include "slab.h"

TEST_CASE("Slab insert/get/release") {
   struct Test {
      int a;
      short b;
   };

   Slab<Test, 256> slab;

   int key1 = slab.alloc_zeroed();
   CHECK(key1 != SIMULO_INVALID_SLAB_KEY);
   int key2 = slab.alloc_zeroed();
   CHECK(key2 != SIMULO_INVALID_SLAB_KEY);

   {
      auto &test1 = slab.get(key1);
      CHECK(test1.a == 0);
      CHECK(test1.b == 0);

      test1.a = 4;
      test1.b = -1;

      auto &test2 = slab.get(key2);
      CHECK(test2.a == 0);
      CHECK(test2.b == 0);
   }

   {
      auto &test = slab.get(key1);
      CHECK(test.a == 4);
      CHECK(test.b == -1);
   }

   slab.release(key1);
   slab.release(key2);
}

TEST_CASE("Slab fill/empty") {
   struct Test {
      int unused;
   };

   {
      Slab<Test, 256> slab;

      for (int i = 0; i < 256; ++i) {
         CHECK(slab.alloc_zeroed() != SIMULO_INVALID_SLAB_KEY);
      }

      for (int i = 0; i < 256; ++i) {
         if (i % 2 == 0) {
            slab.release(i);
         }
      }

      for (int i = 0; i < 256; ++i) {
         if (i % 2 == 0) {
            CHECK(slab.alloc_zeroed() != SIMULO_INVALID_SLAB_KEY);
         }
      }

      for (int i = 255; i >= 0; --i) {
         if (i % 2 == 1) {
            slab.release(i);
         }
      }
   }
}
