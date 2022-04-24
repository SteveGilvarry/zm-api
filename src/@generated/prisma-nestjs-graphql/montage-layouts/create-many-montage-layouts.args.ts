import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsCreateManyInput } from './montage-layouts-create-many.input';

@ArgsType()
export class CreateManyMontageLayoutsArgs {

    @Field(() => [MontageLayoutsCreateManyInput], {nullable:false})
    data!: Array<MontageLayoutsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
