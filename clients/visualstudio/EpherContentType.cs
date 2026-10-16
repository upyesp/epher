// The .epher content type wiring (the "Adding a Language Server
// Protocol extension" recipe): an LSP-based language client loads when
// a file of its content type opens, so .epher needs a content type of
// its own, derived from the code-remote base every LSP client uses.

using Microsoft.VisualStudio.LanguageServer.Client;
using Microsoft.VisualStudio.Utilities;
using System.ComponentModel.Composition;

namespace Epher.VisualStudio
{
    internal static class EpherContentTypeDefinitions
    {
        [Export]
        [Name("epher")]
        [BaseDefinition(CodeRemoteContentDefinition.CodeRemoteContentTypeName)]
        internal static ContentTypeDefinition EpherContentTypeDefinition;

        [Export]
        [FileExtension(".epher")]
        [ContentType("epher")]
        internal static FileExtensionToContentTypeDefinition EpherFileExtensionDefinition;
    }
}
