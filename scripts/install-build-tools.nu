#!/usr/bin/env nu

export def main [
  --make-version: string = '0.36.13'
  --make-arch: string = 'x86_64-unknown-linux-musl'
  --typeshare-version: string = '1.7.0'
  --typeshare-arch: string = 'x86_64-unknown-linux-gnu'
  --sccache-version: string = '0.5.4'
  --sccache-arch: string = 'x86_64-unknown-linux-musl'
  --reset]: nothing -> nothing {
  let local: string = ($env.FILE_PWD | path join '..' '.local' | path expand)
  let cache: string = ($local | path join 'cache')
  let bin: string = ($local | path join 'bin')

  if $reset {
    rm -rf $cache $bin
  }
  
  mkdir $cache $bin
  
  durl make --version=$make_version --arch=$make_arch |
    download --zip ($cache | path join $'cargo-make-($make_version)') |
    binned 'cargo-make' --bin=$bin
  durl typeshare --version=$typeshare_version --arch=$typeshare_arch |
    download --xz ($cache | path join $'typeshare-($typeshare_version)') |
    binned 'typeshare' --bin=$bin 
  durl sccache --version=$sccache_version --arch=$sccache_arch |
    download ($cache | path join $'sccache-($sccache_version)') |
    binned 'sccache' --bin=$bin

  chmod +x ($bin | path join '*')
}

def 'durl make' [
  --version: string
  --arch: string]: nothing -> string {
  durl --org=sagiegurari --repo=cargo-make --tag=$version --asset=$'cargo-make-v($version)-($arch).zip'
}

def 'durl typeshare' [
  --version: string
  --arch: string]: nothing -> string {
  durl --org=1Password --repo=typeshare --tag=$'v($version)' --asset=$'typeshare-cli-v($version)-($arch).tar.xz'
}

def 'durl sccache' [
  --version: string
  --arch: string]: nothing -> string {
  durl --org=mozilla --repo=sccache --tag=$'v($version)' --asset=$'sccache-v($version)-($arch).tar.gz'
}

def durl [
  --org: string
  --repo: string
  --tag: string
  --asset: string]: nothing -> string {
  $'https://github.com/($org)/($repo)/releases/download/($tag)/($asset)'
}

def download [--zip --xz output: string]: string -> string {
  let it = $in

  if ($output | path exists) {
    print $"Already downloaded ($output)"
    return $output
  }

  mkdir $output
  
  if $zip {
    let tmp = ([$output, '.zip'] | str join)
    http get $it | save $tmp
    ^unzip -j $tmp -d $output
    rm $tmp
  } else if $xz {
    http get $it | ^tar xJf - --directory $output --strip-components=1
  } else {
    http get $it | ^tar xzf - --directory $output --strip-components=1
  }

  print $"Downloaded ($it) to ($output)"
  $output
}

def binned [exc: string --bin: string]: string -> nothing {
  let cache = ($in | path join $exc)
  let output = ($bin | path join $exc)

  if ($output | path exists) and ((open $cache | hash sha256) == (open $output | hash sha256)) {
    print $"Already installed ($exc) in ($bin)"
    return
  }

  cp $cache $output
  print $"Installed ($exc) in ($bin)"
}
