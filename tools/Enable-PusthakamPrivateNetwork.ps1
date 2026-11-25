#Requires -RunAsAdministrator
<#
.SYNOPSIS
Allows the selected Pusthakam host on a trusted private local network.
.DESCRIPTION
Changes only the selected active network profile and creates a program/port-scoped
Private-profile TCP rule for the local subnet. Does not remove firewall block rules,
disable the firewall, expose the port on Public networks or configure internet access.
#>
[CmdletBinding(SupportsShouldProcess = $true, ConfirmImpact = 'Medium')]
param(
    [Parameter(Mandatory = $true)]
    [string] $ApplicationPath,
    [ValidateRange(1024, 65535)]
    [int] $Port = 4318,
    [string] $InterfaceAlias = 'Wi-Fi',
    [switch] $TrustedNetwork
)
$ErrorActionPreference = 'Stop'
if (-not $TrustedNetwork) {
    throw 'Use -TrustedNetwork only for a home/library network you trust, never public or guest Wi-Fi.'
}
$application = (Get-Item -LiteralPath $ApplicationPath).FullName
if ((Split-Path -Leaf $application) -notin @('pusthakam.exe', 'pusthakam-server.exe')) {
    throw 'Choose the actual Pusthakam application or console server executable.'
}
$profile = @(Get-NetConnectionProfile -InterfaceAlias $InterfaceAlias)
if ($profile.Count -ne 1) { throw 'Choose exactly one active network interface.' }
if ($profile[0].NetworkCategory -eq 'DomainAuthenticated') {
    throw 'This network is managed by your organisation. Ask its administrator to configure access.'
}
$hasher = [Security.Cryptography.SHA256]::Create()
try { $digest = [BitConverter]::ToString($hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes($application.ToLowerInvariant()))).Replace('-', '').Substring(0, 12) }
finally { $hasher.Dispose() }
$ruleName = "Pusthakam-Browser-$Port-$digest"
Write-Output "Application: $application"
Write-Output "Network: $InterfaceAlias; TCP port: $Port; allowed remote scope: LocalSubnet; profile: Private"
if ($profile[0].NetworkCategory -ne 'Private' -and $PSCmdlet.ShouldProcess($InterfaceAlias, 'Mark this trusted network Private')) {
    Set-NetConnectionProfile -InterfaceIndex $profile[0].InterfaceIndex -NetworkCategory Private
}
if (-not (Get-NetFirewallRule -Name $ruleName -ErrorAction SilentlyContinue)) {
    if ($PSCmdlet.ShouldProcess($ruleName, 'Allow this program and TCP port from the private local subnet')) {
        New-NetFirewallRule -Name $ruleName -DisplayName 'Pusthakam browser access (private local network)' -Direction Inbound -Action Allow -Enabled True -Profile Private -Program $application -Protocol TCP -LocalPort $Port -RemoteAddress LocalSubnet | Out-Null
    }
} else { Write-Output 'The matching Pusthakam browser rule already exists.' }
Write-Output "Firewall rule name: $ruleName"
Write-Output "To remove this rule later: Remove-NetFirewallRule -Name '$ruleName'"
Write-Output "Original network category: $($profile[0].NetworkCategory). Network profile changes affect other applications too."
if (-not $WhatIfPreference) {
    Get-NetIPAddress -InterfaceIndex $profile[0].InterfaceIndex -AddressFamily IPv4 |
        Where-Object { $_.IPAddress -notlike '169.254.*' } |
        ForEach-Object { Write-Output "OPAC: http://$($_.IPAddress):$Port/opac`nStaff: http://$($_.IPAddress):$Port/staff" }
}
