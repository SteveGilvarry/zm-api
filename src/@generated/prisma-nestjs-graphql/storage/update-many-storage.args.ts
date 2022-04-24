import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageUpdateManyMutationInput } from './storage-update-many-mutation.input';
import { StorageWhereInput } from './storage-where.input';

@ArgsType()
export class UpdateManyStorageArgs {

    @Field(() => StorageUpdateManyMutationInput, {nullable:false})
    data!: StorageUpdateManyMutationInput;

    @Field(() => StorageWhereInput, {nullable:true})
    where?: StorageWhereInput;
}
