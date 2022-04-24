import { Injectable } from '@nestjs/common';
import { StorageCreateInput} from '../@generated/prisma-nestjs-graphql/storage/storage-create.input';
import { StorageUpdateInput} from '../@generated/prisma-nestjs-graphql/storage/storage-update.input';
import { StorageWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/storage/storage-where-unique.input';
import { PrismaService } from '../../prisma/prisma.service';

@Injectable()
export class StorageService {
  constructor(private prisma: PrismaService) {}

  create(createStorageInput:StorageCreateInput) {
    return this.prisma.storage.create({
      data: createStorageInput
    });
  }

  findAll() {
    return this.prisma.storage.findMany();
  }

  findOne(storageWhereUniqueInput:StorageWhereUniqueInput) {
    return this.prisma.storage.findUnique({
      where: storageWhereUniqueInput
    });
  }

  update(
    storageWhereUniqueInput:StorageWhereUniqueInput,
    updateStorageInput: StorageUpdateInput
  ) {
    return this.prisma.storage.update({
      where: storageWhereUniqueInput,
      data: updateStorageInput
    });
  }

  remove(storageWhereUniqueInput:StorageWhereUniqueInput) {
    return this.prisma.storage.delete({
      where: storageWhereUniqueInput
    });
  }
}
