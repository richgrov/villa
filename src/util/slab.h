#ifndef SIMULO_UTIL_SLAB_H_
#define SIMULO_UTIL_SLAB_H_

#include "debug_assert.h"

#include <string.h>

#define SIMULO_INVALID_SLAB_KEY -1

template <class T, int Length> struct Slab {
   explicit Slab() : next_available_(0) {
      static_assert(Length > 0);

      for (int i = 0; i < Length; ++i) {
         if (i == Length - 1) {
            objects_[i].next = SIMULO_INVALID_SLAB_KEY;
         } else {
            objects_[i].next = i + 1;
         }
      }
   }

   ~Slab() {
      bool in_use[Length];
      memset(in_use, true, sizeof(in_use));

      int next_available = next_available_;
      while (next_available != SIMULO_INVALID_SLAB_KEY) {
         in_use[next_available] = false;
         next_available = objects_[next_available].next;
      }

      for (int i = 0; i < Length; ++i) {
         if (in_use[i]) {
            release(i);
         }
      }
   }

   [[nodiscard]] T &get(const int index) {
      SIMULO_DEBUG_ASSERT(index >= 0 && index < Length, "get {}, range {}", index, Length);
      return objects_[index].value;
   }

   /**
    * Returns `kInvalidSlabKey` if allocation fails
    */
   int alloc_zeroed() {
      if (next_available_ == SIMULO_INVALID_SLAB_KEY) {
         return SIMULO_INVALID_SLAB_KEY;
      }

      int key = next_available_;
      auto &storage = objects_[key];
      next_available_ = storage.next;
      memset(&storage.value, 0, sizeof(storage.value));
      return key;
   }

   void release(const int key) {
      SIMULO_DEBUG_ASSERT(key >= 0 && key < Length, "release {}, range {}", key, Length);
      objects_[key].next = next_available_;
      next_available_ = key;
   }

   union {
      T value;
      int next;
   } objects_[Length];
   int next_available_;
};

#endif // !SIMULO_UTIL_SLAB_H_
