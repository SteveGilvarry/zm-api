import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageUpdateInput } from './storage-update.input';
import { StorageWhereUniqueInput } from './storage-where-unique.input';

@ArgsType()
export class UpdateOneStorageArgs {

    @Field(() => StorageUpdateInput, {nullable:false})
    data!: StorageUpdateInput;

    @Field(() => StorageWhereUniqueInput, {nullable:false})
    where!: StorageWhereUniqueInput;
}
