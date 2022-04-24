import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageCreateInput } from './storage-create.input';

@ArgsType()
export class CreateOneStorageArgs {

    @Field(() => StorageCreateInput, {nullable:false})
    data!: StorageCreateInput;
}
