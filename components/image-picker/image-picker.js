class ImagePicker extends HTMLElement {
  #imageData = undefined

  get imageData() {
    return this.#imageData
  }

  constructor() {
    super()
    this.shadow = this.attachShadow({ mode: "closed" })
    this.shadow.appendChild(this.#render())
    this.#uploadByClick()
  }

  #render() {
    let template = document.createElement("template")
    template.innerHTML = `
      <link rel="stylesheet" href="/components/image-picker/image-picker.css">
      <div class="data-wrapper">
        <div class="foreground">
          <img src="/components/image-picker/icon/image.svg">
          <p>Pick Emojis</p>
        </div>
        <input class="image_input" type="file" accept=".png,.gif" multiple>
      </div>
    `
    return template.content
  }

  arrayBufferToBase64(buffer) {
    let binary = ''
    let bytes = new Uint8Array(buffer)
    let len = bytes.byteLength
    for (var i = 0; i < len; i++) {
      binary += String.fromCharCode(bytes[i])
    }
    return "data:image/png;base64," + window.btoa(binary)
  }

  #getImageSize(file) {
    return new Promise((resolve, reject) => {
      const img = new Image();
      img.src = URL.createObjectURL(file);

      img.onload = function () {
        resolve({ width: img.naturalWidth, height: img.naturalHeight });
        URL.revokeObjectURL(img.src); // Clean up
      };

      img.onerror = reject; // Handle errors
    });
  }

  #uploadByClick() {
    let myFile = this.shadow.querySelector('.image_input')

    myFile.addEventListener('change', async (e) => {
      let files = e.target.files

      this.#imageData = [];

      if (!files[0]) { return }
      for (let file of files) {
        let img_width, img_height;
        try {
          const { width, height } = await this.#getImageSize(file);
          img_width = width;
          img_height = height;
        } catch (error) {
          console.error(error);
          continue;
        }

        let mime = file.type.split('/')

        let buffer = await file.arrayBuffer()
        let payload = new Uint8Array(buffer)

        let blob = new Blob([payload], { 'type': mime })
        let url = URL.createObjectURL(blob)

        let name = file.name.split('.')
        name.pop()
        name = name.join('.')

        let postData = {
          name: name,
          size: file.size,
          height: img_height,
          width: img_width,
          type: mime[1].charAt(0).toUpperCase() + mime[1].slice(1),
          u8array: Array.from(payload),
          data_url: url
        }

        this.#imageData.push(postData)
      }

      this.dispatchEvent(new Event('change'))
    })
  }
}

export const imagePicker = {
  mount: function () {
    customElements.define('image-picker', ImagePicker)
  },
  unmount: function (index) {
    index
      ? document.querySelectorAll('image-picker')[index].remove()
      : document.querySelector('image-picker').remove()
  }
}