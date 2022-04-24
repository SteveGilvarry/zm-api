import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageWhereUniqueInput } from './storage-where-unique.input';
import { StorageCreateInput } from './storage-create.input';
import { StorageUpdateInput } from './storage-update.input';

@ArgsType()
export class UpsertOneStorageArgs {

    @Field(() => StorageWhereUniqueInput, {nullable:false})
    where!: StorageWhereUniqueInput;

    @Field(() => StorageCreateInput, {nullable:false})
    create!: StorageCreateInput;

    @Field(() => StorageUpdateInput, {nullable:false})
    update!: StorageUpdateInput;
}
