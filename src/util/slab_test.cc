#include <doctest/doctest.h>

#include "slab.h"

TEST_CASE("Slab insert/get/release") {
   int progress = 0;

   class Test {
   public:
      int a_;
      short b_;
      int *progress_;

      Test(int a, short b, int *progress) : a_(a), b_(b), progress_(progress) {
         (*progress_)++;
      }

      ~Test() {
         (*progress_)--;
      }
   };

   simulo::Slab<Test, 256> slab;

   int key1 = slab.emplace(1, 2, &progress);
   CHECK(progress == 1);
   int key2 = slab.emplace(3, 4, &progress);
   CHECK(progress == 2);

   {
      auto &test1 = slab.get(key1);
      CHECK(progress == 2);
      CHECK(test1.a_ == 1);
      CHECK(test1.b_ == 2);
      CHECK(test1.progress_ == &progress);

      test1.a_ = 4;
      test1.b_ = -1;
      test1.progress_ = &progress;

      auto &test2 = slab.get(key2);
      CHECK(progress == 2);
      CHECK(test2.a_ == 3);
      CHECK(test2.b_ == 4);
      CHECK(test2.progress_ == &progress);
   }

   CHECK(progress == 2);

   {
      auto &test = slab.get(key1);
      CHECK(test.a_ == 4);
      CHECK(test.b_ == -1);
   }

   slab.release(key1);
   CHECK(progress == 1);
   slab.release(key2);
   CHECK(progress == 0);
}

TEST_CASE("Slab fill, ctor/dtor") {
   int used = 0;

   class Test {
   public:
      int *used_;

      Test(int *used) : used_(used) {
         (*used_)++;
      }

      ~Test() {
         (*used_)--;
      }
   };

   {
      simulo::Slab<Test, 256> slab;

      for (int i = 0; i < 256; ++i) {
         int key = slab.emplace(&used);
         CHECK(key != simulo::kInvalidSlabKey);
      }
      CHECK(used == 256);

      for (int i = 0; i < 256; ++i) {
         if (i % 2 == 0) {
            slab.release(i);
         }
      }
      CHECK(used == 128);

      for (int i = 0; i < 256; ++i) {
         if (i % 2 == 0) {
            int key = slab.emplace(&used);
            CHECK(key != simulo::kInvalidSlabKey);
         }
      }
      CHECK(used == 256);

      for (int i = 255; i >= 0; --i) {
         if (i % 2 == 1) {
            slab.release(i);
         }
      }
      CHECK(used == 128);
   }

   CHECK(used == 0);
}
