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

static inline int slab_get_next_id(void *obj) {
   return *((int *)obj);
}

static inline void slab_set_next_id(void *obj, int id) {
   *((int *)obj) = id;
}

static inline void slab_init(void *objects, size_t num_objects, size_t obj_size) {
   for (int i = 0; i < num_objects; ++i) {
      void *obj_addr = ((char *)objects) + (obj_size * i);
      if (i == num_objects - 1) {
         slab_set_next_id(obj_addr, SIMULO_INVALID_SLAB_KEY);
      } else {
         slab_set_next_id(obj_addr, i + 1);
      }
   }
}

static inline void slab_reclaim(void *objects, size_t object_size, int obj_id, int *next_avail_id) {
   void *obj_addr = ((char *)objects) + (object_size * obj_id);
   slab_set_next_id(obj_addr, *next_avail_id);
   *next_avail_id = obj_id;
}

#undef SLAB_TYPE
#undef SLAB_LENGTH
#undef SLAB_NAME
