#include "debug_assert.h"

#include <string.h>

#define SIMULO_INVALID_SLAB_KEY -1

struct SLAB_NAME {
   explicit SLAB_NAME() : next_available_(0) {
      static_assert(SLAB_LENGTH > 0);

      for (int i = 0; i < SLAB_LENGTH; ++i) {
         if (i == SLAB_LENGTH - 1) {
            objects_[i].next = SIMULO_INVALID_SLAB_KEY;
         } else {
            objects_[i].next = i + 1;
         }
      }
   }

   ~SLAB_NAME() {
      bool in_use[SLAB_LENGTH];
      memset(in_use, true, sizeof(in_use));

      int next_available = next_available_;
      while (next_available != SIMULO_INVALID_SLAB_KEY) {
         in_use[next_available] = false;
         next_available = objects_[next_available].next;
      }

      for (int i = 0; i < SLAB_LENGTH; ++i) {
         if (in_use[i]) {
            release(i);
         }
      }
   }

   [[nodiscard]] SLAB_TYPE &get(const int index) {
      SIMULO_DEBUG_ASSERT(
         index >= 0 && index < SLAB_LENGTH, "get {}, range {}", index, SLAB_LENGTH
      );
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
      SIMULO_DEBUG_ASSERT(key >= 0 && key < SLAB_LENGTH, "release {}, range {}", key, SLAB_LENGTH);
      objects_[key].next = next_available_;
      next_available_ = key;
   }

   union {
      SLAB_TYPE value;
      int next;
   } objects_[SLAB_LENGTH];
   int next_available_;
};

#undef SLAB_TYPE
#undef SLAB_LENGTH
#undef SLAB_NAME
