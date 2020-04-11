import * as fs from "fs";
import Ajv from "ajv";
import Zip from "adm-zip";
import schema from "./playlist.schema.json";

const ajv = new Ajv();
const validate = ajv.compile(schema);

const pngMagicNumber = new Buffer([
    0x89,
    0x50,
    0x4e,
    0x47,
    0x0d,
    0x0a,
    0x1a,
    0x0a,
]);
const jpgMagicNumber = new Buffer([0xff, 0xd8, 0xff]);

export type BeatmapType = "key" | "hash" | "levelID";

interface JsonPlaylist {
    title: string;
    author?: string;
    description?: string;
    cover?: string;
    maps: JsonBeatmap[];
    customData?: { [key: string]: unknown };
}
interface JsonBeatmap {
    type: BeatmapType;
    date?: string;
    difficulties?: Difficulty[];
    key?: string;
    hash?: string;
    levelID?: string;
    customData?: { [key: string]: unknown };
}

export interface Difficulty {
    name: string;
    characteristic: string;
}

export class Beatmap {
    type: BeatmapType;
    private _date?: string;
    difficulties?: Difficulty[];
    key?: string;
    hash?: string;
    levelID?: string;
    customData?: { [key: string]: unknown };

    private constructor(identifier: string, type: BeatmapType) {
        this.type = type;
        this.date = new Date();
        if (this.type === "key") {
            this.key = identifier;
        } else if (this.type === "hash") {
            this.hash = identifier;
        } else if (this.type === "levelID") {
            this.levelID = identifier;
        } else {
            throw new Error("Unexpected");
        }
    }

    /**
     * Creates a new beatmap identified by its BeatSaver key
     *
     * @param key BeatSaver key
     */
    static newKey(key: string): Beatmap {
        if (!/^[0-9A-Fa-f]{1,8}$/.test(key)) {
            throw new Error("Invalid key");
        }
        return new Beatmap(key, "key");
    }
    /**
     * Creates a new beatmap identified by its hash
     *
     * @param hash Hash
     */
    static newHash(hash: string): Beatmap {
        if (!/^[0-9A-Fa-f]{40}$/.test(hash)) {
            throw new Error("Invalid hash");
        }
        return new Beatmap(hash, "hash");
    }
    /**
     * Creates a new beatmap identified by its level ID
     *
     * @param levelID Level ID
     */
    static newLevelID(levelID: string): Beatmap {
        if (!/^[^\r\n]+$/.test(levelID)) {
            throw new Error("Invalid levelID");
        }
        return new Beatmap(levelID, "levelID");
    }

    get date(): Date | undefined {
        if (this._date === undefined) {
            return undefined;
        } else {
            return new Date(this._date);
        }
    }
    set date(value: Date | undefined) {
        if (value === undefined) {
            this._date = undefined;
        } else {
            this._date = value.toISOString();
        }
    }
}

export class Playlist {
    title: string;
    author?: string;
    description?: string;
    private _coverPath?: string;
    private _coverData?: Buffer;
    maps: Beatmap[] = [];
    customData?: { [key: string]: unknown };

    private _zip: Zip;

    private constructor(title: string);
    private constructor(buffer: Buffer);
    private constructor(arg: string | Buffer) {
        if (typeof arg === "string") {
            this.title = arg;
            this._zip = new Zip();
        } else {
            this.title = "";
            this._zip = new Zip(arg);
        }
    }

    /**
     * Creates a new playlist
     *
     * @param title Title
     * @param author Optionnal author
     * @param description Optionnal description
     */
    static new(title: string, author?: string, description?: string): Playlist {
        const playlist = new Playlist(title);
        playlist.author = author;
        playlist.description = description;
        return playlist;
    }

    /**
     * Read a playlist from disk
     *
     * @param filename Filename
     */
    static async read(filename: string): Promise<Playlist>;
    /**
     * Read a playlist from memory
     *
     * @param buffer Buffer
     */
    static async read(buffer: Buffer): Promise<Playlist>;
    static async read(arg: string | Buffer): Promise<Playlist> {
        let buffer: Buffer;
        if (typeof arg === "string") {
            buffer = await new Promise((res, rej) => {
                fs.readFile(arg, (err, data) => {
                    if (err) {
                        rej(err);
                    }
                    res(data);
                });
            });
        } else {
            buffer = arg;
        }

        const playlist = new Playlist(buffer);

        const playlistFile: string = await new Promise((res, rej) => {
            playlist._zip.readAsTextAsync("playlist.json", (data, err) => {
                if (err) {
                    rej(err);
                }
                res(data);
            });
        });
        const jsonPlaylist: JsonPlaylist = JSON.parse(playlistFile);

        if (!validate(jsonPlaylist)) {
            throw new Error(ajv.errorsText());
        }

        playlist.title = jsonPlaylist.title;
        playlist.author = jsonPlaylist.author;
        playlist.description = jsonPlaylist.description;
        playlist._coverPath = jsonPlaylist.cover;
        playlist.maps = jsonPlaylist.maps.map((m) => {
            let beatmap: Beatmap;
            if (m.type === "key") {
                beatmap = Beatmap.newKey(m.key!);
            } else if (m.type === "hash") {
                beatmap = Beatmap.newHash(m.hash!);
            } else if (m.type === "levelID") {
                beatmap = Beatmap.newLevelID(m.levelID!);
            } else {
                throw new Error("Unexpected");
            }

            if (m.date) {
                beatmap.date = new Date(m.date);
            }
            return beatmap;
        });
        playlist.customData = jsonPlaylist.customData;

        if (playlist._coverPath !== undefined) {
            playlist._coverData = await new Promise((res, rej) => {
                playlist._zip.readFileAsync(
                    playlist._coverPath!,
                    (data, err) => {
                        if (err) {
                            rej(err);
                        }
                        if (!data) {
                            rej(new Error("Unexpected"));
                        }
                        res(data!);
                    }
                );
            });
        }

        return playlist;
    }

    /**
     * Write a playlist to disk
     *
     * @param filename Filename
     */
    async write(filename: string): Promise<void>;
    /**
     * Write a playlist to memory
     */
    async write(): Promise<Buffer>;
    async write(arg?: string): Promise<void | Buffer> {
        const jsonPlaylist: JsonPlaylist = {
            title: this.title,
            author: this.author,
            description: this.description,
            cover: this._coverPath,
            maps: this.maps.map((m) => ({
                type: m.type,
                date: m.date?.toISOString(),
                difficulties: m.difficulties,
                key: m.key,
                hash: m.hash,
                levelID: m.levelID,
                customData: m.customData,
            })),
            customData: this.customData,
        };

        if (!validate(jsonPlaylist)) {
            throw new Error(ajv.errorsText());
        }

        if (jsonPlaylist.cover !== undefined) {
            this._zip.addFile(jsonPlaylist.cover!, this._coverData!);
        }

        const buffer = this._zip.toBuffer();
        if (typeof arg === "string") {
            return await new Promise((res, rej) => {
                fs.writeFile(arg, buffer, (err) => {
                    if (err) {
                        rej(err);
                    }
                    res();
                });
            });
        } else {
            return buffer;
        }
    }

    get cover(): Buffer | undefined {
        return this._coverData;
    }
    set cover(value: Buffer | undefined) {
        if (value === undefined) {
            this._coverPath = undefined;
            this._coverData = undefined;
        } else {
            if (value.slice(0, pngMagicNumber.length) === pngMagicNumber) {
                if (this._coverPath !== undefined) {
                    this._zip.deleteFile(this._coverPath);
                }

                this._coverPath = "cover.png";
                this._coverData = value;
            } else if (
                value.slice(0, jpgMagicNumber.length) === jpgMagicNumber
            ) {
                if (this._coverPath !== undefined) {
                    this._zip.deleteFile(this._coverPath);
                }

                this._coverPath = "cover.jpg";
                this._coverData = value;
            } else {
                throw new Error("Invalid cover data");
            }
        }
    }
}
