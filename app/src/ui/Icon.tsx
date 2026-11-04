import archive from "pixelarticons/svg/archive.svg?raw";
import arrowLeft from "pixelarticons/svg/arrow-left.svg?raw";
import arrowRight from "pixelarticons/svg/arrow-right.svg?raw";
import check from "pixelarticons/svg/check.svg?raw";
import copy from "pixelarticons/svg/copy.svg?raw";
import folder from "pixelarticons/svg/folder.svg?raw";
import globe from "pixelarticons/svg/globe.svg?raw";
import laptop from "pixelarticons/svg/laptop.svg?raw";
import link from "pixelarticons/svg/link.svg?raw";
import plus from "pixelarticons/svg/plus.svg?raw";
import reload from "pixelarticons/svg/reload.svg?raw";
import server from "pixelarticons/svg/server.svg?raw";
import sliders from "pixelarticons/svg/sliders.svg?raw";
import wifi from "pixelarticons/svg/wifi.svg?raw";

const icons = { archive, arrowLeft, arrowRight, check, copy, folder, globe, laptop, link, plus, reload, server, sliders, wifi };

export type IconName = keyof typeof icons;

// The svgs ship with the app and come straight from the package, so
// injecting them as markup is fine here.
export function Icon({ name, size = 20 }: { name: IconName; size?: number }) {
  return (
    <span
      aria-hidden="true"
      style={{ display: "inline-flex", width: size, height: size }}
      dangerouslySetInnerHTML={{ __html: icons[name].replace("<svg ", `<svg width="${size}" height="${size}" `) }}
    />
  );
}
