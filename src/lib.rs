use crate::savefile::SaveFile;

pub mod savefile;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let str = r#"
    {
  "targets": [
    {
      "isStage": true,
      "name": "Stage",
      "variables": {
        "`jEk@4|i[#Fk?(8x)AV.-my variable": [
          "my variable",
          0
        ]
      },
      "lists": {},
      "broadcasts": {},
      "blocks": {},
      "comments": {},
      "currentCostume": 0,
      "costumes": [
        {
          "assetId": "cd21514d0531fdffb22204e0ec5ed84a",
          "name": "backdrop1",
          "md5ext": "cd21514d0531fdffb22204e0ec5ed84a.svg",
          "dataFormat": "svg",
          "rotationCenterX": 240,
          "rotationCenterY": 180
        }
      ],
      "sounds": [
        {
          "assetId": "83a9787d4cb6f3b7632b4ddfebf74367",
          "name": "pop",
          "dataFormat": "wav",
          "format": "",
          "rate": 44100,
          "sampleCount": 1032,
          "md5ext": "83a9787d4cb6f3b7632b4ddfebf74367.wav"
        }
      ],
      "volume": 100,
      "layerOrder": 0,
      "tempo": 60,
      "videoTransparency": 50,
      "videoState": "on",
      "textToSpeechLanguage": null
    },
    {
      "isStage": false,
      "name": "Sprite1",
      "variables": {},
      "lists": {},
      "broadcasts": {},
      "blocks": {
        "}j@q:Jl+%6t6(4ZJK35N": {
          "opcode": "motion_movesteps",
          "next": "f7=2Lg;)pL}cp0e;fd|C",
          "parent": null,
          "inputs": {
            "STEPS": [
              1,
              [
                4,
                "0"
              ]
            ]
          },
          "fields": {},
          "shadow": false,
          "topLevel": true,
          "x": 323,
          "y": 153
        },
        "f7=2Lg;)pL}cp0e;fd|C": {
          "opcode": "motion_movesteps",
          "next": null,
          "parent": "}j@q:Jl+%6t6(4ZJK35N",
          "inputs": {
            "STEPS": [
              1,
              [
                4,
                "1"
              ]
            ]
          },
          "fields": {},
          "shadow": false,
          "topLevel": false
        }
      },
      "comments": {},
      "currentCostume": 0,
      "costumes": [
        {
          "assetId": "b7853f557e4426412e64bb3da6531a99",
          "name": "costume1",
          "bitmapResolution": 1,
          "md5ext": "b7853f557e4426412e64bb3da6531a99.svg",
          "dataFormat": "svg",
          "rotationCenterX": 48,
          "rotationCenterY": 50
        },
        {
          "assetId": "e6ddc55a6ddd9cc9d84fe0b4c21e016f",
          "name": "costume2",
          "bitmapResolution": 1,
          "md5ext": "e6ddc55a6ddd9cc9d84fe0b4c21e016f.svg",
          "dataFormat": "svg",
          "rotationCenterX": 46,
          "rotationCenterY": 53
        }
      ],
      "sounds": [
        {
          "assetId": "83c36d806dc92327b9e7049a565c6bff",
          "name": "Meow",
          "dataFormat": "wav",
          "format": "",
          "rate": 44100,
          "sampleCount": 37376,
          "md5ext": "83c36d806dc92327b9e7049a565c6bff.wav"
        }
      ],
      "volume": 100,
      "layerOrder": 1,
      "visible": true,
      "x": 0,
      "y": 0,
      "size": 100,
      "direction": 90,
      "draggable": false,
      "rotationStyle": "all around"
    }
  ],
  "monitors": [],
  "extensions": [],
  "meta": {
    "semver": "3.0.0",
    "vm": "0.2.0-prerelease.20200625173937",
    "agent": "Mozilla/5.0 (X11; Linux x86_64; rv:68.0) Gecko/20100101 Firefox/68.0"
  }
}
"#;
        let s: SaveFile = serde_json::from_str(&str).unwrap();
        println!("{}", serde_json::to_string(&s).unwrap());
    }
}
