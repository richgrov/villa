#ifndef SIMULO_UTIL_SLAB_H_
#define SIMULO_UTIL_SLAB_H_

#include "debug_assert.h"

#include <algorithm>
#include <string.h>
#include <utility>

#define SIMULO_INVALID_SLAB_KEY -1

template <class T, int Length> class Slab {
public:
   explicit Slab() : next_available_(0) {
      static_assert(Length > 0);

      for (int i = 0; i < Length; ++i) {
         if (i == Length - 1) {
            get_storage(i).next = SIMULO_INVALID_SLAB_KEY;
         } else {
            get_storage(i).next = i + 1;
         }
      }
   }

   ~Slab() {
      bool in_use[Length];
      memset(in_use, true, sizeof(in_use));

      int next_available = next_available_;
      while (next_available != SIMULO_INVALID_SLAB_KEY) {
         in_use[next_available] = false;
         next_available = get_storage(next_available).next;
      }

      for (int i = 0; i < Length; ++i) {
         if (in_use[i]) {
            release(i);
         }
      }
   }

   [[nodiscard]] T &get(const int index) {
      return get_storage(index).value;
   }

   /**
    * Returns `kInvalidSlabKey` if allocation fails
    */
   int alloc_zeroed() {
      if (next_available_ == SIMULO_INVALID_SLAB_KEY) {
         return SIMULO_INVALID_SLAB_KEY;
      }

      int key = next_available_;
      auto &storage = get_storage(key);
      next_available_ = storage.next;
      memset(&storage.value, 0, sizeof(storage.value));
      return key;
   }

   void release(const int key) {
      get_storage(key).next = next_available_;
      next_available_ = key;
   }

private:
   union Storage {
      T value;
      int next;
   };

   Storage &get_storage(const int index) {
      SIMULO_DEBUG_ASSERT(
         index >= 0 && index < Length, "index {} out of slab range {}", index, Length
      );
      return objects_[index];
   }

   Storage objects_[Length];
   int next_available_;
};

#endif // !SIMULO_UTIL_SLAB_H_
