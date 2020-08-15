# dbml-language-server

This is a Language server for DBML (DBML-LS) that adheres to the [Language Server Protocol (LSP)](https://github.com/Microsoft/language-server-protocol/blob/master/protocol.md).

The DBML-LS is server that runs in the background, providing IDEs, text editors and other tools with information about DBML schemas. Since LSP servers are frontend independent, they work with any editors with bindings to LSP.

Please note that this is very much a POC at the moment, and some features are still missing, such as goto definition and renaming, but semantic code completion is working with a limited set of features.

Available features are better shown with a GIF:

![semantic_completion_example_gif](https://user-images.githubusercontent.com/17864887/90304148-fece2f80-de8a-11ea-99dd-1f0710c18941.gif)

## Missing features: ##

### Major ###

- Goto Definition;
- Renaming;

#### Completion: ####

- Inline relationships;
- Composite foreign keys;
- Indexes, and indexes settings;
- Relationships attributes and settings.

Binaries will soon be available.
