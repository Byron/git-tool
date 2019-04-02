#
# Module manifest for module 'Git-Tool'
#
# Generated by: Benjamin Pannell
#
# Generated on: 01/04/2019
#

@{

    # Script module or binary module file associated with this manifest.
    RootModule        = 'Git-Tool.psm1'

    # Version number of this module.
    ModuleVersion     = '1.2'

    # Supported PSEditions
    # CompatiblePSEditions = @()

    # ID used to uniquely identify this module
    GUID              = 'fcffa40d-9731-4913-8ac3-3c770fa4ddf6'

    # Author of this module
    Author            = 'Benjamin Pannell'

    # Company or vendor of this module
    CompanyName       = 'Sierra Softworks'

    # Copyright statement for this module
    Copyright         = '2019 Sierra Softworks. All rights reserved.'

    # Description of the functionality provided by this module
    Description       = 'Simplify checking out your Git repositories in a structured directory space'

    # Minimum version of the Windows PowerShell engine required by this module
    # PowerShellVersion = ''

    # Name of the Windows PowerShell host required by this module
    # PowerShellHostName = ''

    # Minimum version of the Windows PowerShell host required by this module
    # PowerShellHostVersion = ''

    # Minimum version of Microsoft .NET Framework required by this module. This prerequisite is valid for the PowerShell Desktop edition only.
    # DotNetFrameworkVersion = ''

    # Minimum version of the common language runtime (CLR) required by this module. This prerequisite is valid for the PowerShell Desktop edition only.
    # CLRVersion = ''

    # Processor architecture (None, X86, Amd64) required by this module
    # ProcessorArchitecture = ''

    # Modules that must be imported into the global environment prior to importing this module
    # RequiredModules = @()

    # Assemblies that must be loaded prior to importing this module
    # RequiredAssemblies = @()

    # Script files (.ps1) that are run in the caller's environment prior to importing this module.
    # ScriptsToProcess = @()

    # Type files (.ps1xml) to be loaded when importing this module
    # TypesToProcess = @()

    # Format files (.ps1xml) to be loaded when importing this module
    # FormatsToProcess = @()

    # Modules to import as nested modules of the module specified in RootModule/ModuleToProcess
    NestedModules     = @()

    # Functions to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no functions to export.
    FunctionsToExport = @(
        "Get-Repo",
        "Get-RepoInfo",
        "New-Repo",
        "Open-Repo",
        "Get-CurrentRepo",
        "Get-GitIgnore",
        "Get-Repos",
        "Get-RepoNamespaces",
        "Set-DevDirectory",
        "Get-DevDirectory"
    )

    # Cmdlets to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no cmdlets to export.
    CmdletsToExport   = @()

    # Variables to export from this module
    VariablesToExport = @("GitTool")

    # Aliases to export from this module, for best performance, do not use wildcards and do not delete the entry, use an empty array if there are no aliases to export.
    AliasesToExport   = @()

    # DSC resources to export from this module
    # DscResourcesToExport = @()

    # List of all modules packaged with this module
    # ModuleList = @()

    # List of all files packaged with this module
    # FileList = @()

    # Private data to pass to the module specified in RootModule/ModuleToProcess. This may also contain a PSData hashtable with additional module metadata used by PowerShell.
    PrivateData       = @{

        PSData = @{

            # Tags applied to this module. These help with module discovery in online galleries.
            # Tags = @()

            # A URL to the license for this module.
            LicenseUri = 'https://sierrasoftworks.com/licenses/MIT'

            # A URL to the main website for this project.
            ProjectUri = 'https://github.com/SierraSoftworks/git-tool'

            # A URL to an icon representing this module.
            IconUri    = 'https://cdn.sierrasoftworks.com/logos/icon.png'

            # ReleaseNotes of this module
            # ReleaseNotes = ''

        } # End of PSData hashtable

    } # End of PrivateData hashtable

    # HelpInfo URI of this module
    HelpInfoURI       = 'https://github.com/SierraSoftworks/git-tool/issues'

    # Default prefix for commands exported from this module. Override the default prefix using Import-Module -Prefix.
    # DefaultCommandPrefix = ''

}

