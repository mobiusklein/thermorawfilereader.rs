# Why there is a copy of FlatBuffers here?

Version: 23.5.26

C# allows you to define compile-time symbols to conditionally generate different code. Unfortunately, you cannot set these symbols on imported references.

The `Google.FlatBuffers` library has several symbols that allow it to use raw memory access instead of forcing you to make additional copies to export the buffer in memory. To save some time moving memory around fewer times, I had to vendor a copy of the `FlatBuffers` code and set the constants myself.