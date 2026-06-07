import { BlenderBuildKind, MeasurementUnitKind } from "..";
import { DAILY_LOWERCASE, PATCH_LOWERCASE, RELEASE_LOWERCASE } from "../../constants";

export const fromStrBlenderBuildTypeKind = (input: string) => {
    if (input !== undefined) {
        switch (input!.toLowerCase()) {
            case RELEASE_LOWERCASE: {
                return BlenderBuildKind.Release;
            }
            case DAILY_LOWERCASE: {
                return BlenderBuildKind.Daily;
            }
            case PATCH_LOWERCASE: {
                return BlenderBuildKind.Patch;
            }
            default: {
                return BlenderBuildKind.Release;
            }
        }
    }
}

export const MeasurementUnitKindName: Record<MeasurementUnitKind, string> = {
    [MeasurementUnitKind.BIT]: "bit",
    [MeasurementUnitKind.BYTE]: "byte",

    [MeasurementUnitKind.KB]: "kb",
    [MeasurementUnitKind.MB]: "mb",
    [MeasurementUnitKind.GB]: "gb",
    [MeasurementUnitKind.TB]: "tb",
    [MeasurementUnitKind.PB]: "pb",
    [MeasurementUnitKind.EB]: "eb",
    [MeasurementUnitKind.ZB]: "zb",
    [MeasurementUnitKind.YB]: "yb",

    [MeasurementUnitKind.KIB]: "kib",
    [MeasurementUnitKind.MIB]: "mib",
    [MeasurementUnitKind.GIB]: "gib",
    [MeasurementUnitKind.TIB]: "tib",
    [MeasurementUnitKind.PIB]: "pib",
    [MeasurementUnitKind.EIB]: "eib",

    [MeasurementUnitKind.BPS]: "bps",
    [MeasurementUnitKind.BpS]: "bps",

    [MeasurementUnitKind.KBS]: "kbs",
    [MeasurementUnitKind.KBpS]: "kbps",

    [MeasurementUnitKind.MBS]: "mbs",
    [MeasurementUnitKind.MBpS]: "mbps",

    [MeasurementUnitKind.GBS]: "gbs",
    [MeasurementUnitKind.GBpS]: "gbps",

    [MeasurementUnitKind.TBS]: "tbs",
    [MeasurementUnitKind.TBpS]: "tbps",

    [MeasurementUnitKind.KIBS]: "kibs",
    [MeasurementUnitKind.KIBpS]: "kibps",

    [MeasurementUnitKind.MIBS]: "mibs",
    [MeasurementUnitKind.MIBpS]: "mibps",

    [MeasurementUnitKind.GIBS]: "gibs",
    [MeasurementUnitKind.GIBpS]: "gibps",

    [MeasurementUnitKind.NS]: "ns",
    [MeasurementUnitKind.US]: "us",
    [MeasurementUnitKind.MS]: "ms",
    [MeasurementUnitKind.S]: "s",

    [MeasurementUnitKind.MIN]: "min",
    [MeasurementUnitKind.H]: "h",
    [MeasurementUnitKind.D]: "d",
    [MeasurementUnitKind.W]: "w",
    [MeasurementUnitKind.M]: "m",
    [MeasurementUnitKind.Y]: "y",

    [MeasurementUnitKind.VER]: "ver",
    [MeasurementUnitKind.SER]: "ser",
};
