# dbml-language-server

This is a Language server for DBML (DBML-LS) that adheres to the [Language Server Protocol (LSP)](https://github.com/Microsoft/language-server-protocol/blob/master/protocol.md).

The DBML-LS provides a server that runs in the background, providing IDES, text editors and other tools with information about DBML schemas.

Currently, it only supports code completion.

Expected features are:
- Goto Definition;
- Renaming;

Features available are better shown with a GIF:

![semantic_completion_example](https://user-images.githubusercontent.com/17864887/90304148-fece2f80-de8a-11ea-99dd-1f0710c18941.gif)


The DBML-LS is frontend-independent.

This is very much a POC at the moment, and some features are still missing, such as goto definition and renaming.

Expect binaries soon.
