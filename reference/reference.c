// reference implementation from Philip Koopman
// from https://users.ece.cmu.edu/~koopman/crc/book/Chapter7_examples.c

// Copyright 2024 Philip Koopman
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//    http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#include "chapter7_examples.h"

static const uint32_t initialSeed = SEED;  // initial sum

// ----------------------
// Example from section 7.7.2 
// 8-bit Koopman checksum
// dataWord: data word in 8-bit blocks
// dwSize:   number of bytes in the data word
// modulus:  should be set to 253, or possibly 239
//   Only uses bottom 16 bits of sum

uint8_t Koopman8B(uint8_t dataWord[],
                     uint32_t dwSize, uint32_t modulus) 
{ 
  assert((modulus == 253) || (modulus == 239));
  assert(dwSize > 0);
  assert(initialSeed <= 0xFF);
  
  uint32_t sum = dataWord[0] ^ initialSeed;

  for(uint32_t index = 1; index < dwSize; index++)
  { sum = ((sum<<8) | dataWord[index]) % modulus;
  }

  // Append implicit zero
  sum = (sum<<8) % modulus;
  return((uint8_t)sum);
}


// ----------------------
// Example from section 7.7.3 
// 8-bit Koopman checksum with 24-bit blocks
// dataWord: data word in 8-bit bytes, zero padded
// dwSize:   number of bytes in the data word
// modulus:  should be set to 253

uint8_t Koopman8W(uint8_t dataWord[],
                     uint32_t dwSize, uint32_t modulus) 
{ 
  assert((modulus == 253) || (modulus == 239));
  assert(dwSize > 0);
  assert(initialSeed <= 0xFF);

  uint32_t sum = dataWord[0] ^ initialSeed;
  uint32_t index = 1;
  
  // Process three bytes at a time
  const uint32_t blockSize = 3;
  while(index < (dwSize-blockSize) )
  { uint32_t threeBytes =
                  ( (uint32_t)dataWord[index+2])
                | (((uint32_t)dataWord[index+1])<<8)
                | (((uint32_t)dataWord[index+0])<<16);
    sum = ((sum<<24) | threeBytes) % modulus;
    index += blockSize;
  }
  
  // Process any remaining bytes, one byte at a time
  while(index < dwSize)
  { uint32_t oneByte = (uint32_t)dataWord[index]; 
     sum = ((sum<<8) | oneByte) % modulus;
    index++;
  }
  
  // Append implicit zero
  sum = (sum<<8) % modulus;  
  return((uint8_t)sum);
}


// ----------------------
// Example from section 7.7.4 
// 16-bit Koopman checksum
// dataWord: data word in 8-bit bytes, zero-padded
// dwSize:   number of bytes in the data word
// modulus:  should be set to 65519

uint16_t Koopman16W(uint8_t dataWord[],
                     uint32_t dwSize, uint32_t modulus) 
{ 
  assert((dwSize & 1) == 0); // Even number of bytes
  assert(dwSize > 1);        // At least two bytes
  assert(modulus == 65519);

  // Special-case the first block to initialize sum
  uint32_t sum =        (uint32_t)dataWord[1] 
        | ((initialSeed^(uint32_t)dataWord[0]) << 8);

  // Process rest of blocks, two at a time
  const uint32_t blockSize = 2;
  for(uint32_t index = 2; index < dwSize; 
                                  index += blockSize)
  { uint32_t oneBlock =  
                  (uint32_t)dataWord[index+1] 
              | (((uint32_t)dataWord[index+0])<<8);
    sum = ((sum<<16) + oneBlock) % modulus;
  }
  
  // Append implicit zeros
  sum = (sum<<16) % modulus;
  return((uint16_t)sum);
}


// ----------------------
// Example from section 7.7.5 
// 16-bit Koopman checksum, computed a byte at a time
// dataWord: data word in 8-bit bytes
// dwSize:   number of bytes in the data word
// modulus:  should be set to 65519
//   Only uses bottom 16 bits of sum

uint16_t Koopman16B(uint8_t dataWord[],
                     uint32_t dwSize, uint32_t modulus) 
{ 
  assert(modulus == 65519);
  assert(dwSize > 0);
  assert(initialSeed <= 0xFF);

  uint32_t sum = initialSeed ^ dataWord[0];

  for(uint32_t index = 1; index < dwSize; index++)
  { sum = ((sum<<8) + dataWord[index]) % modulus;
  }

  // Append two bytes of implicit zeros
  sum = (sum<<8) % modulus;
  sum = (sum<<8) % modulus;
  return((uint16_t)sum);
}


// ----------------------
// Example from section 7.7.6 
// 32-bit Koopman checksum
// dataWord: data word in 8-bit bytes, zero-padded
// dwSize:   number of bytes in the data word
// modulus:  should be set to 4294967291
//    Note: Uses all 64 bits of the sum variable
//          for intermediate results
//    Note: Must be an integral number of 4-byte blocks

uint32_t Koopman32W(uint8_t dataWord[],
                     uint32_t dwSize, uint32_t modulus) 
{ 
  assert(dwSize > 3);
  assert((dwSize & 3) == 0); // divisible by 4 bytes
  assert(modulus == 4294967291);
  assert(initialSeed <= 0xFF);
 
  uint64_t sum = initialSeed<<24;

  uint32_t oneBlock =    (uint32_t)dataWord[3] 
                     | (((uint32_t)dataWord[2])<<8)
                     | (((uint32_t)dataWord[1])<<16)
                     | (((uint32_t)dataWord[0])<<24);
  sum ^= (uint64_t)oneBlock;

  const uint32_t blockSize = 4;
  for(uint32_t index = 4; index < dwSize; 
                                  index += blockSize)
  { oneBlock =  (uint32_t)dataWord[index+3] 
            | (((uint32_t)dataWord[index+2])<<8)
            | (((uint32_t)dataWord[index+1])<<16)
            | (((uint32_t)dataWord[index+0])<<24);
    sum = ((sum<<32) + (uint64_t)oneBlock) % modulus;
  }
  
  // Append four bytes of implicit zeros
  sum = (sum<<32) % modulus; 
  return((uint32_t)sum);
}


// ----------------------
// Example from section 7.7.7 
// 32-bit Koopman checksum, computed a byte at a time
// dataWord: data word in 8-bit bytes
// dwSize:   number of bytes in the data word
// modulus:  should be set to 4294967291
//    Note: Needs a sum 5 bytes in size to handle 
//            intermediate sum values
uint32_t Koopman32B(uint8_t dataWord[],
                     uint32_t dwSize, uint32_t modulus) 
{ 
  assert(dwSize > 1);
  assert(modulus == 4294967291);
  assert(initialSeed <= 0xFF);
  
  uint64_t sum = initialSeed ^ dataWord[0];

  for(uint32_t index = 1; index < dwSize; index++)
  { sum = ((sum<<8) + dataWord[index]) % modulus;
  }

  // Append four bytes of implicit zeros
  sum = (sum<<32) % modulus; 
  return((uint32_t)sum);
}

