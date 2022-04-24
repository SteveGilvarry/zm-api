import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigCreateManyInput } from './config-create-many.input';

@ArgsType()
export class CreateManyConfigArgs {

    @Field(() => [ConfigCreateManyInput], {nullable:false})
    data!: Array<ConfigCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
