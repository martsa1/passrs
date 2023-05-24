#[cfg(test)]
mod test {
    use anyhow::Result;
    use pgp::base64_reader::Base64Reader;

    const KEYPHRASE: &str = "sample";

    const SAMPLE_CONTENT: &str = concat!(
        "4gG2y&9?-]AAE(wUnD]v22zs\"nx}ad\n",
        "---\n",
        "username: sample@example.com\n",
        "some_key: foobar",
    );

    const SAMPLE_ENTRY_B64: &str = concat!(
        "hQIMAzqA30aahEXUAQ//UtDdpDyeoHPXi1Wci6bPSUOJ5fsU+tZicpiea052rYc8ZWG5UbiVteru",
        "Bl8fqJvSmtUX41/rgf8l9jT6jHUVPI0/4zFmVJuK5onSM7beErGTeqIrvSKpqZow/TurNXivLW5b",
        "HCoZD5YXtz1SLu5o+mDu3zzichGzCxN72IdAe8PVmhmMh+TP/G2EyT70Zx7P7ZcFC6vnoOCZAhUA",
        "E0nwQgeo9R7C044H/DIETwVNSRLUKshL/p7r+gzdmyaW5hB/l93DLNKnHZIlH46WG8BW0xlJjZKK",
        "rZ/64UVIotTLKqNUX9oMcAvEGdoLei1mYHqoAAkODNGT7MxopQqYLj6wPkugb76sCDeZ5NAYZtk3",
        "npDLxJCxtlmz9hScTvDyQUaZM8sUflkslpjkRW6WQq3O1ulfi1syvLjdXmTc8uttvGNRUq3XRODv",
        "44Wd7uexPNgpFPdFLdreQkEMkwJm6qVlPckAO89PSMb+XqD9vMyJMSaa6pG4HnHiD6UtJZkG2jvX",
        "fDiFuGUpfwq9Qc0avqp9HOugUEUGvSH9MYPdykhH81gkgQJgHpPJHoQu5qGO2ArTe5it5l00pxyF",
        "CGPf3beugly693ejVAuoJVmNNGotN41nVdrKFhSYsua1V/rfUNWlDfUQSzBt6Ft0VvPBJ8Gsr6oo",
        "NjNAg1lHi7lPHTgh7YHSoAE7KK3q7ADw9ypTkbaXZL7yHibOFR7lFLZiVVwQQ5/FImy3FuzGkq9C",
        "JFqnO97bvxOPBtlsoi+3eK/P1ovDsaRIdMOUvbA4d2097OxO+QR4B2wrFGrQPhizDCbN3X27JGrf",
        "J4fzluGJDIVzjF1XQfnB+0jBfKF84g3AU9939b5iUtxcK8n91p5W0QAd8sX0jXQPKhplX6U9RMGj",
        "n/6DBXc=",
    );

    const SAMPLE_ARMOURED_KEY_B64: &str = concat!(
        "LS0tLS1CRUdJTiBQR1AgUFJJVkFURSBLRVkgQkxPQ0stLS0tLQoKbFFkR0JHUnQwRUFCRUFESzZD",
        "WE91QlNhUjFpUjJZcjVtR3haazBXRXRYdG9MTm5DL05ObnpqVjRsY20rUTdSWAplcEtZRS9VbnIx",
        "UWlYWi8rL0FEVXUyZEo5cFFyZm4rR1hLZ0tSczFUQXpweE5jNFR3NDRINGNVUjVnVVBCOTNQCis4",
        "K1VkSmJRY1Q5ZzdnNzljby9XYVV3WVFmVFd0V2drWWxOV1ZlSitORkhoMFBvSWFMdXlCcHBxcEFO",
        "MnllQ1AKQndjaGUvZDZjRFM0M2o2MC9tQTlBeEdoZk5FbTVuQUJleHlaL0YwMHRhUGhsL1Z0b2pK",
        "T25NcVF1WUNJRlhraAovK3llNXBESXkyeEZqTzVzOXNSMDcvZlA3Z2U3NG5IYXhtbU5rZDlIZ2VJ",
        "WFF0NjFzdUdTK0QwanVMeUowR1ZVCnZWNFlubnlMWUR1eGJwMmFWeHRiYU1WU2Z5bWFFaEVmNHJC",
        "OTJJdytCRXJQanJObUJhTUxpTUU4RVBxZExtUjUKVkRtc3BDUW5CSWp6Mm54R29FaGJhbURuRmh0",
        "S1hGTjdIOGdzV1RWUFdXZ1EwUUN4aXVXSHpjMlcvWmhlMWlmeApYNmRpTHhHTXhpWnBoUjB2aVY2",
        "dThzVUhOOXJoUzVGL29kOXBGUjFneHh4Vnd2VDZadzZWZC9oY1VLaXgvd2JRCkNjRlJ4bDdmbGsy",
        "SDZYaGlxU0Zmd3hHMnFpQ1Y5bXpkTlN5MS9idXdvencxM1NlR0g1N2NqdFhNY2d0ZEg0QS8KNExX",
        "WjN1RXRta3RJVlU5M1RBUnFnYVh3MmFVMnM5VE8vL29CVnhnMDQ4VkcxMjc5TVNyeXF1VWswNllL",
        "Y01nZAp3djlqMVVPZENvT1Z6eEppQlQrSmh5RXhCQmxMUmN0TlJnOUs3eXoxNWQ1TmdrblNvTW1O",
        "MkxPcExRQVJBUUFCCi9nY0RBdEw5ZnNmVnZ5dnkvNFVtQmpIQ3R3YVI4ODhUYktuZkVIdUFzWE4z",
        "VUpMTktuL3NCaUJXaGo2V0hoU24KemlKOTQvUkJZYjU5Uzdyc0J3VTJNZkt1ZGNvcFRNL3dCZE02",
        "WmZnQzk1bzhxSVJOY3k5Q3V5RXlRcEQ5Vm5XYQpUcVVDTWVLOURJc1lnNGx4KzZMeHE3bndaRVFW",
        "aWdZa0FpbHh1ZWtlajB3a2h6VHhkdEtCdW9uUU5Cbyt5ODgwClVGRmtCMDZXK3hnSkRzTkl1eGd5",
        "TkFuZXFkYWo0SGpFWnEyeVNzKzk0bmFrZWRna1U2QjBPeFNGWWRtWDZKWFkKeU1NOE9mOElyam9M",
        "UG4vWWk3R2xJVUZRblc1NGJwajg0QVgya0pKaENFaGRHOHNYK2NZMmFkQS9ZSkJVZlNWRwpKaXMz",
        "Nk1KZWE0RW9PVjZOUm1xNHlNblhzaXpRUVY0cFFRZ1dZeDRKQlBSK0FZcG9MdWZwMDBRdVVkVW00",
        "N2pRCkVncFFGYS9mRGVERlp2OUJtTU1XK3VjZ3dySnJ2SzlKRkJTSW5uczlHTGdqalRWTFQyd3U3",
        "aW90MkRkbU5RM04KNnNORWVOSmpVdnVYQmhDdEZZaGxnczQraHBZZkhEeG5FbFpNS2hoU3hUYXNy",
        "Vmp5Z280Uk00TlRMS0Y4dnNaNgo4bzQ0Y1h5d2s4Z2NoTm84WXNUbzVHdXk4UEEwdlFxM0ZmSG9H",
        "RnFZZEJsQjNwRE5vQzZLVm9RK3ZHdnNjbWJGCjVHTmZPbE9tSTlvMmZBN0pxZ2lOZ1V4M1JSdWNW",
        "MjlvekYrQkx0N3htV0RGaWpNVnlWNmJBNjB0MGNxd21aKy8KSnc3Uml5UGpORXV6TGtGTEFYOHBS",
        "dW5xOU5HbjcyakN2OEVxbkc0WkhHMlZRQk9sR1ViRVJJcWI2UkVtWWt6eQpCSVRIaXBlN2o3T1Qr",
        "WC9JcUd0S3pXV2orZTVkVmN4VnNDZFVMcmhkTGdvVnM2VFJQU1paSWgwOElhMTFtbm8vClllNGF1",
        "Y0tqNWF3WWFlc0VnaFRmS1h2SXRBTWhJQjdWdGxMUWsyRGxkaHRKVzdabFhKbUJ4Q0JnbGtCeHY3",
        "ZkcKM0p3aGlKYk9OMGpEUkhvdFhvdlhRU2lwMElGWkY1aUN0YzhGSTVtMjlQMG5QM253RXVuenZy",
        "M1dBeTJHdGY1Swp3bmk3Sms1NWticFpzazJRaTRraDEwREl4dmRyamEyc0U0ZzBEaU54cjhTMEsx",
        "VzRHay85WjlnVU1xMzJPZVhMClRIYW5mU2JtRlNDdGllUDJBa0N1aHhTeDF5WEdqWWZJRmNqVFZw",
        "U0x2T0VqSEZMMytBZko3NitMYzRqbTduekEKY2JVVlpEOTNQNWFxcit4dG1TWGVMUnRXcHZvY2VH",
        "WlRnMUE4M2tYelVmNHV5Tk1UdkREUlB3OGprdDhTRHVRUwpXa0I2Ny8vYkNoOExoaU9vaGp4dnFB",
        "ampHTXYwZERLZ0ZRYTlITlRMRkdpRFkrS0ZyVmtXRDFuZXhmTkVYSTQ5CmZHWFdzbFBNZUZQUVg4",
        "NHdtdERlN1hzNHlQRm9MTk9MMk90ZUNlNDJGeUxpVS93cmYwWEUrbnU3MUZud0ZPd04KQVdVVG13",
        "MkxPMUoxNmNGNmVCK1BFZk9WUGtyV08xcW0rZWNtSlpyRjExTmJMREQ0RFVUS3ZqRy9VeVFVb3kz",
        "Kwo4MDdEbzViZUlkQzl4UHE4d2U2eUROS2M1NjA3VmNBSlhGcFI4bXA3dWNuaTFrTVVLWWZrMXdF",
        "SFBCa01yUENkClAxaW1hWllQTXRtRnNsbXhXREdVRGVWWU5VRlpiU0RrN2hjTHlEWEtlY0IrN05k",
        "WUp6bk9LU2RqQmtNVGo0QzMKRnlCcFA2L1IyeEl4TENROENrcFZkQlhLNUVoM0pEKzFwZUJVd2hW",
        "c2xiVTEzVWxoa2lNdFQ5c3lNQ0doNVdBZQo3alYrQytRQTF6cTlVcUVmVHJRcytBVnBVMzIxVzM1",
        "bHk2K09UaEp0Y3FDWGVrbjNDY0RvVGx3N3FVbmFQQ01uCkowRDZkUVU4dHpSOFNSbXF5eERCRVZw",
        "U3Z6QTZmSFJ2M3RYbDFxbjZBdXFPNHk0Tkw4WjlhMnFzOUJMT2NHQzAKWjRyZnM2M1lyK2t4cDNO",
        "dnB4UUpiNnFRa2IzYzZkMDI3MDRZVEVIaVoreHpXRUZhRVZoTkhqa1NIR0NjWkRTQgpYMHZJQjZm",
        "emZ1UG8xamtMMUZtdlRzUDAyT3lSeTJtR01ydmVjaG43MitBd0VoVVhOQ2thZmZKZ0ZGMTlOcVhs",
        "CjdST2JuU1l5K2xPUVZhbkNWNVpneWRTVVRFYkVpazZOTjhTbnpibGxUdk9CODM3K1JYc2JTTHkw",
        "SkhOaGJYQnMKWlNBb2MyRnRjR3hsS1NBOGMyRnRjR3hsUUdWNFlXMXdiR1V1WTI5dFBva0NUQVFU",
        "QVFvQU5oWWhCS0wvTmx3LwpmR1p3RWEzVHV2Y1JJeUlaMzJXVEJRSmtiZEJBQWhzREJBc0pDQWNF",
        "RlFvSkNBVVdBZ01CQUFJZUJRSVhnQUFLCkNSRDNFU01pR2Q5bGs0K3ZELzk2YktVWWlVOVVuN3dh",
        "cHpKbWNxNUg2VlRtM2tLVWlsWi9qOUlmeEc5OWdmMXQKVldQZlV4QWVrQVc5bGtYa1llTHVRdnRv",
        "dFpTeXpuVzVCUUNWRkQ2U0NJZm81cjJWQmhTVTVrTldCaUgvR3lXUAovSTIrTmdXOXJnYkMxNVQx",
        "OVJIYzNCNGk3a2NreUlFY1hPWXBvb240YnMvTi82OHZJV2xkckxmUkNXWWJ3Rm51CldadENkNk5B",
        "T2hzZnk4OGJ4dVlSdiswOGtZWEtUT3hITGRUbXJkZEpBcUdDWVRmRGQwQkxhaG5ZVTlBNTNhbTQK",
        "eUJXemJkUGxpd1J0Q3FmYVZCWmpwMTlvOXk4dFNBcmZzVmYxNzNhaG5QS0dlL3ZmN2dvUXQvcXZW",
        "UFF5T2xZNApDSUFBenZFSjVETXB5aUJmdGVjYmhzSFVBYVBGTTk1R3hoYmxoRXpBY1FVaEp0L1VZ",
        "Z21vNGlMdTN5VnNXeWVmCnFPZUViOUdyM0hFSDc3UnNzcXBtZHZ2dGdlL0FZbnoxemU1SkNmR2U0",
        "WnVUWFhqQTR2c1IzOVVyUlZUZ0l4MXYKTUV4TGpXWUpyVVB4QjgzdG9pS2lLdy80c1hTRUlaZGwx",
        "VDU2VGRUd0twNy82R05hOU5qcWgzOWFsT3BqWittcApIYXV1alRkZWJaUVFmbzlSamE0VjdSc3RS",
        "TVBzUGlqK3FtMjY5SERTa0pGekRLNTM1Z1BwaElXWStRcDNDZmVGCklCRm54Q1dkcUc3bUZJdndi",
        "cU9CT2FvRWJURXpyNW1mN2c4VTJ0bTljQTc3NUZLYXpHenhqb1Y5Z2xyTXUxSHUKbUtKK1hTTTlm",
        "ZG9wL0NCeGJENVk3QktOWjQzNkxXVHZUb0FUQlQvUnRWeHE3L3kveGdobVRNR3dJczYxaUowSApS",
        "Z1JrYmRCQUFSQUF3dVE2Z2xDSmJFMytTc0xRSmNiNUIxWHhSK0FLL1FSL2dLcG0vZ2ZVVWEybGJF",
        "NzhJMTRRCk54eC9ZZzB6aEFIeFJtaExiSldTWGZkRUpNeFJKdjZycmIxeTd1VWNnK2NRbXNYbXVl",
        "Qks2Q2NsWGJzeE1SNnQKbjY4YnVCVDlLL0daRUhEcXdsendyZTZYd1hBRHBHcy9MM210d1hHRHpG",
        "ZHRQTWcvRjZDdnNKbkZGNGt1dEtGRQpZYjVlTUp2YnVKUnB3RzZiRWRBZDNhTDlGOTBReXZLcUFL",
        "SUdsR25oSVNJc3Z6SXAreEJUN3NlUDkyQzhJcm9wCmd2UzZhc0o5aVhWSjdhSGsxMWdRaDZYbGoy",
        "QmZGRWlNV0ZWaC81UVhDNDZRZVl6ZXZPOVovdkt1M3ptb3p5VXoKQk8vdmFrYU9FUC9nOUMvdGFI",
        "bndWc00ybllYRkhYRFJYTHNRekhQTk80cFlwdFlkZXNpNWZTZlFqTWh5NmlONwpMcTE4NmhXOG9D",
        "eE9oakRtVWtPVklaWDduOW9nNHo3dXdrTVBORFp1UkJrNTNNYjJRTmd4a1U4ZjEyZDI5eHpOCnZ0",
        "cFVaNlJZdjladjQ5M0pHTDZBN0VvRWF2STIzLzlRQ2N6ckkzb2FZdnNnZTRTWVEvRTNJNDBadFd3",
        "SmR2ZXIKWG8rRDB5UytvdVJ0UUN0T0Z0bUFOVVpYbW1VNXdjSnYzSmp1Y1dNRVlYVGVhYTRDamJC",
        "ZnRvTjlwMXQwWVVjNgphS09YeCtrbFhxZGxKajQ5U2Y5ZWlxbGZkcE5VeGp0T21lS1d1UzRJaC9I",
        "RFZhdkV0NHlzZ0pXME41WG9HSlg1CmRhWmpXbDVYMzFlem00OE11TTVoSEE0WVMrYmFJZGJaa3RL",
        "Z29DTk16d2JaVTR4T2lEMURaRzhBRVFFQUFmNEgKQXdMMXRHczNnN3N2WFArU0RSdVM3aCtRY2Y0",
        "VkxXaHhjL1JPR3dBK1FNa05SeHZLUzZKbCs5VXhIT3hjOU92bwpVZXlJRkh1NUthV1BrZ0hocGdz",
        "aEJhN0xreDBvdE9OSjduY1lzOTJseVFyTisxc1BsOTFqSTVhWDhnK2hDTFB4CkRNZ1Q1d3FGcTN3",
        "OFlFYmhIVlEwRGNiczI2ZmRuSVFNSUJOWFNtTWZQL215VUIrRHE1aC84cmFWYjM5MVBZUEQKbFd2",
        "aEJDMzhqZlZpZSsyNHU2WXc0K3hQaEFsQkJUYk4xM3JNUUhNR1Rua3gxbXppVzc1SjZjTHIwK3I0",
        "a0NJQwpxeTR2V2EwVzdLQzRVbU9lQmNMeXNpQXNUdnZJNncwUHVwY1RLY2pBbVlGT0U0U003UkhG",
        "aWFxRXhMVkZDUUNuCldUQ3VkR1AwWDM5VFg0M1NiNWF6ZmV2dTVoU0QvbTE1Y3B4all5MnR4MXY4",
        "amFvMk1JNW5TeDRpdURrNVRmVlcKOE1Cd1BtNm44UDhJWHRlam1EL0E1T0tNdThOTUEzMHdKa1Mw",
        "dVBZUEF6Q2w3bEd0NHlPV2tqbXk1dXBaZ0RVZQpJSWZwUEJZTzFodVc4c05RdWR2WHZEcHJ4anMr",
        "THdJZlQ4Tm5MRXJpK0d2NXQ1a2RacEZWWU9sYUtiUU81S2VvClJWbHQ4QlF3aFgwcXB4cS93b0lj",
        "dFB1cE5JOGYwVjJaRVJBdFpxeUJZVjVSYnBEa0pPNVRxRHhBTlFHekZoNGUKUUlXaGpaZEY3VThy",
        "ZVFQTVQ5OUppd1p2N1p3dTBHaThveTRRWEsvZ2JmaklkVjRoVmZVb1ArKzRqQWZJNEJyMQo1UVMv",
        "NUthR3RLL0l4MzcxenRBZXZBSCsyOFhOZkttbkpuOHNaRm5xQ2Z2cjdoZ214SWh3eDduSWdtUHBG",
        "TnZlClhuMm45WXBHQzBacGJqTWNENW1TKytWTDAweDBNZ0xBK0JFc2RTc1pRRDNqWXJZazRTNGRD",
        "WWNiNW1XS2Y1M2MKcmhIQkJ0RS95QVFwQWY5NktvQVVFbTlTM1hYK1U2SVo4QzZqNmxlUmE1Mits",
        "MlVmaXdER0FhUUJkenQwcFJWOQpZM3RHQ1J3Y2JIc0dPMGRENzhzQmVtTTFVUzgxWlA0bTlUQktq",
        "aWVzTjNSR0tWeFlxRTJCLzY3VWowV0p5ZWRpCjVjNnJEWjJvc0t1aHU2TEtleHpjVEczOVo5WnlY",
        "aWZvZnNhSFRkVDZEclMxU1U3MEtCZ0MvN0k2bjdPRzM3RzcKd1RVZ1BCZisrZ1lGUVVlZ05wYTZ1",
        "SFhSTU50RE1EMUNMZ2ZFc3ExK2E1azdVM2kvVzBCTXZxaHcrY3hjcVFuUwpNRkFFQ3E0UzlEMVp0",
        "eGxmRFRNUitXa2cwWThVMEljN3FrckYxK2YySGhxeFBkUXVIakdYWDJyS0hKYUNWdkg4CjFJOTBq",
        "M2NURXJJblRhSHFxQk04SnBqRGNtYkpNa1I4STZaUUFZc1BlT0ZBcVlvUWhsTVExcmlMRW5mNkdU",
        "SzQKWW1ocWdXNkJ4WVpibkllTjQ4b2R3R3NQTDFCVXFUT3YxSldJMXFxZjNnQnFJeDJ4TXNBRzhY",
        "TDc1U3VsMVBBUQp4RkMyL2NPV3c1K3FPRUdGZ1RTdVA3b2IxOEF4TlZJcTZicTV6WUhKT014VWlz",
        "d01lWjlsSmsxanVxVmhmTmlmClpKTyswVTJzTnRYRHd0MGRlOHdGZTd6andMenc2K1FqaTFIS1RO",
        "WXp3M2NpRGMvakhZR0g3UGZwamVHdHdnb04KRTBuK0tFNXA2alFBazUvaHRtYlpnT1dTS25BNGk1",
        "VzByRTRPZ3pvZEEyU3FDUGlOTUYwV1dwalRYK3RlaGFyTwprY3Zrdm50SjRORFRyc00xVk9wS3hn",
        "KzFvV1dXUTRkanQ3R0NtblU3Y2E2R2VySWlBWjE5ejVXOGliOFp2YVozCkVBMnVTdFo0c0t4RFJj",
        "KzhzZmRGWHZaeXhiL0swY1RmbFpXeHByNzZEbDc5M3o4SFBHUzMwSXdpYXJnakZJckIKY1l6Rk1Y",
        "T05hYmlJNUl4SGRxUHNUWmg5NWYzODJDNmhraXRPSGgwVmwzR3d5NW5DQ1p6TjY3VmM4Y1J0ZTdi",
        "VAp1MUE5OUxXRVYrYjZaeitnSS84T0JIcEtUdlhCLzA3NktOeDZmYld4YWIydWlicVdTNHZkZTBL",
        "aXQ3ZDJQaHFsCllWQTJOVXZYYk9IeDdUblRCd1FBQlRlZ0ZPNGVtVlZzVk5QUXkyTFdYays1M3Ex",
        "cnJBK2VHOUc0MzFFVHlFNnEKdFVXaTJ0bTRJM3RESXFoVW5ybEovV1d4NDc0T2hnY3lDbWloUmRI",
        "V3g5bjljNjZ5c2tzMWlRSTJCQmdCQ2dBZwpGaUVFb3Y4MlhEOThabkFScmRPNjl4RWpJaG5mWlpN",
        "RkFtUnQwRUFDR3d3QUNna1E5eEVqSWhuZlpaUHhhdy85CkVKaGR5QVErSlFWZ0dTVmd3ZmZjaldy",
        "MGRUamo0RUxRYzc4ZFR2U0FlYjJ0U3ZxNzY5MDIrRHFsdDFHZ0tIamMKc3h2VndOL3h3K05qbFJr",
        "bndQeUJoUW9BWFIzUzI3bGowN2RMTWVKd09zUjE2OFRwWWRhNVg4MWdSME9hbnhBcgoyd3VOS2ph",
        "SS9VZy94Y2RyajFhMTViVmRpOStpQmRTL3pCc0VBL3pGOHB1UEYvOTBRRGltbXAySnkzMVFCUEFQ",
        "Cng2U1U1b3JONFJLTVYwUVhad0JVWFBMSEJMTzBNR3B2T0R4cjc3U2FvemNWenFoZ0RDYllhK2FM",
        "Mzg1dVo1Y3gKTWFqTDkxQkpSa3hZQnQ3bEN2ODhRQ3dZUWxRRmFSTXowc2Z5ZW1aTkNvZnpkVm0x",
        "ZStDQjZZRmZqb0tmblFpOApWNkI3MWNjN1Qvc0VEaDJOU2hOZGJtQXpkTWxyTHFPeXVvUVlUWUF1",
        "d0RJRXZqT2tvVjgzbThmQWoxVXRQWVByCkw0aFhSc1pxM2lWb3NMdTh5Q1hjOXFNVWhWSitOb1pO",
        "bG9odC8xdjlIVjZQY2tSUUhkeGlKeXE2cmxEZmFUUTEKWG9IKzZFZmhyK1VSVElyT1NBQXhVNDlw",
        "YnEyKzAxN1FYbWpodjl5MXVzb2k1VXBkbjZkbnNkdE0wbnhWdHBkVQp3SG9icm8yMVpIZ29RNjJC",
        "cTVRR0lmbHFBVFRwamNnUWc4SVpGbHFpOTdram9pdnlFVHNnK1VSaUJtakVFWlpDCjNBK1FNclVk",
        "Q20vVCtkK1NMb3pob1Y0K0c4NzFXd1pLQ1lXV2wzM3FNSGt3N1FTSlBlWHJSSGpWT0xtMHZQcGcK",
        "b0Y1RW9Ydmp5V2cwV2ZJT2JXTHVwNThnOEFWSlVUdEd3TDhUeTV1ZVR6UT0KPTdPRmYKLS0tLS1F",
        "TkQgUEdQIFBSSVZBVEUgS0VZIEJMT0NLLS0tLS0K",
    );

    #[test]
    fn test_check_b64_decode() -> Result<()> {
        use base64::prelude::{Engine as _, BASE64_STANDARD};
        let decode_res = BASE64_STANDARD.decode(SAMPLE_ARMOURED_KEY_B64);

        assert!(
            !decode_res.is_err(),
            "decode_res was not Ok: {:?}",
            decode_res,
        );

        Ok(())
    }
}