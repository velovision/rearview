# Power Management

## Minimal Quiescent Power

A powered off raspberry pi still consumes a non-negligible amount of power.

Rearview I/O board version 1.4 introduces some additional components to actually cut off power to the raspberry pi when shut off.

A MAX16054 latching push-button controller represents the 'desired' state. Then, it is the responsibility of `supreme-server` to detect what that 'desired' state is and shut down the pi.

