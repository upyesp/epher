package io.github.upyesp.epher.eclipse;

import java.util.List;

import org.eclipse.lsp4e.server.ProcessStreamConnectionProvider;

/**
 * Launches the shared {@code epher-lsp} server over stdio, found on
 * PATH — the same bring-your-own-binary contract as the emacs, vim,
 * and neovim clients (ADR-0066): the user installs the binary from
 * the release page once; the plugin never downloads anything.
 *
 * GUI-launched Eclipses on macOS do not inherit the shell PATH; there,
 * start Eclipse from a terminal or symlink the binary into
 * /usr/local/bin (see the client README).
 */
public final class EpherConnectionProvider extends ProcessStreamConnectionProvider {

	public EpherConnectionProvider() {
		setCommands(List.of("epher-lsp"));
		setWorkingDirectory(System.getProperty("user.dir")); //$NON-NLS-1$
	}

	@Override
	public String toString() {
		return "epher language server (epher-lsp on PATH)"; //$NON-NLS-1$
	}
}
