import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigCreateManyInput } from './config-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyConfigArgs {

    @Field(() => [ConfigCreateManyInput], {nullable:false})
    @Type(() => ConfigCreateManyInput)
    data!: Array<ConfigCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
