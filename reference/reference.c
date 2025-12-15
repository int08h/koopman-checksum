A PPENDIX A: E XAMPLE CODE
The below C code fragments are intended to illustrate the
key idea behind the use of Koopman checksums. They are
written in a way to make the key ideas obvious. They are not
intended as an illustration of portability or otherwise-desirable
code structure.
Variable typing is per <cstdint>, <stdint.h>, or a similar
definition approach [C++]. The variable dwSize is assumed to
be the number of relevant elements in an 8-bit data word array,
although any size block can be processed so long as the
intermediate sum variable is large enough to hold the next block
plus the current modulo result without overlap.
A. Koopman8 checksum
uint8_t Koopman8(uint8_t dataWord[],
uint32_t dwSize, uint32_t modulus)
{
assert((modulus == 253) || (modulus == 239));
assert(dwSize > 0);
assert(initialSeed <= 0xFF);
uint32_t sum = dataWord[0] ^ initialSeed;
for(uint32_t index = 1; index < dwSize; index++)
{
sum = ((sum<<8) | dataWord[index]) % modulus;
}
// Append implicit zero
sum = (sum<<8) % modulus;
return((uint8_t)sum);
}
B. Koopman16 checksum; 8-bit blocks
uint16_t Koopman16(uint8_t dataWord[],
uint32_t dwSize, uint32_t modulus)
{
assert(modulus == 65519);
assert(dwSize > 0);
assert(initialSeed <= 0xFF);
uint32_t sum = initialSeed ^ dataWord[0];
for(uint32_t index = 1; index < dwSize; index++)
{
sum = ((sum<<8) + dataWord[index]) % modulus;
}
// Append two bytes of implicit zeros
sum = (sum<<8) % modulus;
sum = (sum<<8) % modulus;
return((uint16_t)sum);
}
C. Koopman16P checksum
This implementation assumes there is a function “Parity()”
which returns a 1-bit parity value of the input.
uint16_t Koopman16P(uint8_t dataWord[],
uint32_t dwSize, uint32_t modulus)
{
assert(modulus == 32749);
assert(dwSize > 0);
assert(initialSeed <= 0xFF);
uint32_t sum = initialSeed ^ dataWord[0];
uint32_t psum = sum; // Initialize parity sum
for(uint32_t index = 1; index < dwSize; index++)
{
sum = ((sum<<8) + dataWord[index] ) % modulus;
psum ^= dataWord[index];
}
// Append two bytes of implicit zeros
sum = (sum<<16) % modulus;
// Pack sum with parity
sum = (sum<<1) | Parity((uint8_t)psum);
// Append parity as bottom bit of check value
return((uint16_t)sum);
}
D. Koopman32 checksum
uint32_t Koopman32(uint8_t dataWord[],
uint32_t dwSize, uint32_t modulus)
{
assert(dwSize > 1);
assert(modulus == 4294967291);
assert(initialSeed <= 0xFF);
uint64_t sum = initialSeed ^ dataWord[0];
for(uint32_t index = 1; index < dwSize; index++)
{
sum = ((sum<<8) + dataWord[index]) % modulus;
}
// Append four bytes of implicit zeros
sum = (sum<<32) % modulus;
return((uint32_t)sum);
}
13
E. Koopman32P checksum
This implementation assumes there is a function “Parity()”
which returns a 1-bit parity value of the input.
uint32_t Koopman32P(uint8_t dataWord[],
uint32_t dwSize, uint32_t modulus)
{
assert(dwSize > 1);
assert(modulus == 0x7FFFFFED);
uint64_t sum = initialSeed ^ dataWord[0];
uint32_t psum = (uint32_t)sum; // Initialize parity sum
for(uint32_t index = 1; index < dwSize; index++)
{
sum = ((sum<<8) + (uint64_t)dataWord[index] )
% modulus;
psum ^= dataWord[index];
}
// Append four bytes of implicit zeros
sum = (sum<<32) % modulus;
// Pack sum with parity
sum = (sum<<1) | Parity((uint8_t)psum);
// Append parity as bottom bit of check value
return((uint32_t)sum);
}
