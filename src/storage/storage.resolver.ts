import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { StorageService } from './storage.service';
import { StorageCreateInput} from '../@generated/prisma-nestjs-graphql/storage/storage-create.input';
import { StorageUpdateInput} from '../@generated/prisma-nestjs-graphql/storage/storage-update.input';
import { StorageWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/storage/storage-where-unique.input';

@Resolver('Storage')
export class StorageResolver {
  constructor(private readonly storageService: StorageService) {}

  @Mutation('createStorage')
  async create(
    @Args('createStorageInput') createStorageInput: StorageCreateInput
  ) {
    const created = await this.storageService.create(createStorageInput);
  }

  @Query('storage')
  findAll() {
    return this.storageService.findAll();
  }

  @Query('storage')
  findOne(@Args('id') Id: number) {
    return this.storageService.findOne({ Id });
  }

  @Mutation('updateStorage')
  update(
    @Args('id') Id: number,
    @Args('updateStorageInput') updateStorageInput: StorageUpdateInput
  ) {
    return this.storageService.update({ Id }, updateStorageInput);
  }

  @Mutation('removeStorage')
  remove(@Args('id') Id: number) {
    return this.storageService.remove({ Id });
  }
}
