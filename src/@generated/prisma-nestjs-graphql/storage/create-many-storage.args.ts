import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { StorageCreateManyInput } from './storage-create-many.input';

@ArgsType()
export class CreateManyStorageArgs {

    @Field(() => [StorageCreateManyInput], {nullable:false})
    data!: Array<StorageCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
