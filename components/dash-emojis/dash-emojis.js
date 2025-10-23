import { apiCall } from '/resource/js/api-call-v1.0.js';

class DashEmojis extends HTMLElement {
  #selected_images = [];
  #available_emojis_count = 0;
  #uploading = false;

  constructor() {
    super()
    this.shadow = this.attachShadow({ mode: "closed" });
    this.shadow.appendChild(this.#render())

    this.#addImageSelectorListener()
    this.#addUploadListener()
    this.#getAvailableEmojis()
  }

  #spinnerLoader = `
    <svg class="spinner" viewBox="0 0 50 50">
      <circle class="path" cx="25" cy="25" r="20" fill="none" stroke-width="4"></circle>
    </svg>
  `
  #dotLoader = `
    <div class="dot-loader-wrapper">
      <span class="dot"></span>
      <span class="dot"></span>
      <span class="dot"></span>
      <span class="dot"></span>
    </div>
  `

  /**
    * @brief renders basic layout of the component
    */
  #render() {
    let template = document.createElement('template')
    template.innerHTML = `
    <link rel="stylesheet" href="/resource/css/reset-v1.0.css">
    <link rel="stylesheet" href="/components/dash-emojis/dash-emojis.css">
    <div class="data-wrapper">
      <div class="image-selector">
        <image-picker></image-picker>
        <div class="upload-button">
          <img src="/components/dash-emojis/icon/upload.svg">
        </div>
      </div>
      <div class="selected-image-wrapper">

      </div>
      <h6 class="available_emoji_title">Available Emojis ()</h6>
      <div class="available_emoji_container">

      </div>
    </div>
    `
    return template.content
  }

  async #getAvailableEmojis() {
    let response = await apiCall({
      method: "GET",
      apiUrl: "/api/emoji/list"
    })

    if (!response.ok) {
      uniman.toast.setNotification({
        type: 'error',
        message: response.message
      });
    }
    else {
      let emojiTitle = this.shadow.querySelector('.available_emoji_title');
      emojiTitle.innerText = `Available Emojis (${response.data.length})`;
      this.#available_emojis_count = response.data.length;

      let emojiContainer = this.shadow.querySelector('.available_emoji_container');
      for (let emoji of response.data) {
        let emojiCard = document.createElement('div');
        emojiCard.classList.add('item');
        emojiCard.innerHTML = `
          <div class="image">
            <img src="/emoji/${emoji.uuid}">
          </div>
          <p>${emoji.name}</p>
        `

        emojiContainer.appendChild(emojiCard);
      }
    }
  }

  #addUploadListener() {
    let uploadButton = this.shadow.querySelector('.upload-button');

    uploadButton.addEventListener('click', async (event) => {
      event.preventDefault();

      for (let image of this.#selected_images) {
        if (Math.round(image.size / 1024) > 128) {
          uniman.toast.setNotification({
            type: 'error',
            message: `Image size is too large. Max size is 128kb`
          });

          return;
        }

        if (Math.round(image.width) > 256 || Math.round(image.height) > 256) {
          uniman.toast.setNotification({
            type: 'error',
            message: `Image dimension is too large. Max dimension is 256x256`
          });

          return;
        }
      }

      this.#uploadEmojis();
    })
  }

  async #uploadEmojis() {
    if (this.#uploading) return;

    let selectedImagesInfos = this.shadow.querySelectorAll('.selected-image-wrapper .item');
    if (selectedImagesInfos.length != this.#selected_images.length) {
      uniman.toast.setNotification({
        type: 'error',
        message: `Something went wrong getting the images data`
      });

      return;
    }

    let selected_images = [];
    for (let i = 0; i < this.#selected_images.length; i++) {
      let name = selectedImagesInfos[i].querySelector('.name').innerText;
      let serial_number = selectedImagesInfos[i].querySelector('.serial_number').innerText;;
      if (!name) {
        uniman.toast.setNotification({
          type: 'error',
          message: `Please enter name for the emoji ${i + 1}`
        });
        return
      }
      else if (!serial_number) {
        uniman.toast.setNotification({
          type: 'error',
          message: `Please enter the serial number for the emoji ${i + 1}`
        });
        return;
      }

      selected_images.push({
        name: name,
        serial: Number(serial_number),
        data: this.#selected_images[i].u8array,
        type: this.#selected_images[i].type
      })
    }

    this.#uploading = true;
    let uploadBtn = this.shadow.querySelector('.upload-button');
    uploadBtn.innerHTML = this.#spinnerLoader;

    let response = await apiCall({
      method: "POST",
      apiUrl: "/api/emoji/add",
      reqBody: {
        emojis: selected_images
      }
    })

    if (!response.ok) {
      uniman.toast.setNotification({
        type: 'error',
        message: response.message
      });
    }
    else {
      let selectedImageContainer = this.shadow.querySelector('.selected-image-wrapper');
      selectedImageContainer.innerHTML = '';
      uniman.toast.setNotification({
        type: 'success',
        message: response.message
      });

      this.#getAvailableEmojis();
    }

    this.#uploading = false;
    uploadBtn.innerHTML = `<img src="/components/dash-emojis/icon/upload.svg">`;
  }

  #addImageSelectorListener() {
    let imageUploader = this.shadow.querySelector('image-picker');
    imageUploader.addEventListener('change', (event) => {
      let images = imageUploader.imageData;
      this.#selected_images = [...this.#selected_images, ...images];

      this.#renderSelectedImages();
    })
  }

  #renderSelectedImages() {
    let selectedImageWrapper = this.shadow.querySelector('.selected-image-wrapper');
    selectedImageWrapper.innerHTML = '';

    for (let i = 0; i < this.#selected_images.length; i++) {
      let image = this.#selected_images[i];
      let item = document.createElement('div');
      item.classList.add('item')
      item.innerHTML = `
        <img class="image" src="${image.data_url}">
        <div class="info">
          <span class="serial_number" contenteditable="true">${this.#available_emojis_count + i + 1}</span>
          <span class="name" contenteditable="true">${image.name}</span>
          <div class="size">
            <img src="/components/dash-emojis/icon/${Math.round(image.size / 1024) > 128 ? 'cross' : 'okay'}.svg">
            <p>${Math.round(image.size / 1024)}kb / 128kb</p>
          </div>
          <div class="pixel">
            <img src="/components/dash-emojis/icon/${Math.round(image.width) > 256 || Math.round(image.height) > 256 ? 'cross' : 'okay'}.svg">
            <p>${Math.round(image.width)}px x ${Math.round(image.height)}px / 256px x 256px</p>
          </div>
        </div>
        <img src="/components/dash-emojis/icon/delete.svg" class="remove" onclick="this.getRootNode().host.removeImage(${i})">
      `

      selectedImageWrapper.appendChild(item);
    }
  }

  removeImage(index) {
    this.#selected_images.splice(index, 1);
    this.#renderSelectedImages();
  }
}

export const dashEmojis = {
  mount: function () {
    customElements.define('dash-emojis', DashEmojis)
  },
  unmount: function (index) {
    index
      ? document.querySelectorAll('dash-emojis')[index].remove()
      : document.querySelector('dash-emojis').remove()
  }
}