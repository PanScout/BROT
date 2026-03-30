## Pointers

* We have two pointers singleton, array.
* Singleton
    1 word that points to one word.
* Array 
    * 2 word, reading it causes a special sequence in the FSM. We load each into the special pointer registers.
* Pointer register 
    * 13 total 82 bits each. Singletons fit the lower 41 wasted space but whatever
* Pointers contain RW bit
    * 0 = Read only
    * 1 = RW
* No NULL pointers. Pointers must point to a valid memory address
* We enforce single owner multiple readers at the OS level
    * We also keep a ledger at the OS level,  every pointer with RW access must have no overlaping magisteria. 
* We create 3 tag tpes new
    * Pointer Array
    * Pointer End
    * Pointer Singleton
* Pointer arrays in memory are organzied as Pointer_Array | Pointer_End
* We create pointers through a make pointer instruction, converting an integer to pointer. Thus to setup an array pointer we define two ints, set them, and do make pointer array and make pointer end. We also set the rw bit through the instruction.
* We thus have two modes of accessing/reading memory, load/store with immedaite offset and register offset. In each case it does a bounds check.
* Singletons need no bounds check and you CANNOT do load/store with any offset, just the thing it points to. Thus only load immeadtie with 0 offset is allowed for them.
