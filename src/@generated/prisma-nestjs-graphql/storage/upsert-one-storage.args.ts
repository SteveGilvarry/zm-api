import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageWhereUniqueInput } from './storage-where-unique.input';
import { Type } from 'class-transformer';
import { StorageCreateInput } from './storage-create.input';
import { StorageUpdateInput } from './storage-update.input';

@ArgsType()
export class UpsertOneStorageArgs {

    @Field(() => StorageWhereUniqueInput, {nullable:false})
    @Type(() => StorageWhereUniqueInput)
    where!: StorageWhereUniqueInput;

    @Field(() => StorageCreateInput, {nullable:false})
    @Type(() => StorageCreateInput)
    create!: StorageCreateInput;

    @Field(() => StorageUpdateInput, {nullable:false})
    @Type(() => StorageUpdateInput)
    update!: StorageUpdateInput;
}
